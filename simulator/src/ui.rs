use std::{io::Stdout, time::Duration};

use color_eyre::eyre::Result;
use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Cell, List, ListItem, Paragraph, Row, Table},
};
use tokio::sync::{mpsc, watch};

use crate::types::{AppSnapshot, SimCommand};

pub fn render_ui(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    snapshot: &AppSnapshot,
) -> Result<()> {
    let paused = snapshot.paused;

    terminal.draw(|f| {
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints(
                [
                    Constraint::Length(3),
                    Constraint::Min(5),
                    Constraint::Length(10),
                ]
                .as_ref(),
            )
            .split(f.area());

        let header = Paragraph::new(vec![
            Line::from(vec![
                Span::raw("Node: "),
                Span::styled(
                    snapshot
                        .node_pk
                        .map(|pk| format!("{pk}"))
                        .unwrap_or_else(|| "unknown".into()),
                    Style::default().fg(Color::Cyan),
                ),
                Span::raw("  Delegate: "),
                Span::styled(
                    format!("{}", snapshot.delegate_pk),
                    Style::default().fg(Color::Green),
                ),
                Span::raw("  Paused: "),
                Span::styled(
                    format!("{}", paused),
                    Style::default().fg(if paused { Color::Yellow } else { Color::Green }),
                ),
            ]),
            Line::from(vec![
                Span::raw("Tx sent/ok/err: "),
                Span::styled(
                    format!(
                        "{}/{}/{}",
                        snapshot.total_sent, snapshot.total_ok, snapshot.total_err
                    ),
                    Style::default().fg(Color::Magenta),
                ),
                Span::raw("  Inflight: "),
                Span::styled(
                    snapshot.inflight.to_string(),
                    Style::default().fg(Color::Yellow),
                ),
                Span::raw("  Last err: "),
                Span::styled(
                    snapshot.last_failure.as_deref().unwrap_or("-"),
                    Style::default().fg(Color::Red),
                ),
            ]),
        ])
        .block(Block::default().borders(Borders::ALL).title("Status"));
        f.render_widget(header, layout[0]);

        let body_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(60), Constraint::Percentage(40)].as_ref())
            .split(layout[1]);

        let mut rows = Vec::new();
        for wallet in snapshot.wallets.iter() {
            let row_style = Style::default().fg(wallet_color(wallet.id));
            let mut balance_lines = Vec::new();
            for (asset, balance) in snapshot.assets.iter().zip(wallet.balances.iter()) {
                let short_policy = &asset.policy_id_hex[0..8];
                balance_lines.push(Line::from(vec![
                    Span::styled(asset.name, Style::default().add_modifier(Modifier::BOLD)),
                    Span::raw(format!(
                        " ({short_policy}) bal={} notes={}",
                        balance.balance, balance.notes
                    )),
                ]));
            }

            let balances_cell = Cell::from(Text::from(balance_lines.clone()));
            let height = balance_lines.len().max(1) as u16;

            rows.push(
                Row::new(vec![
                    Cell::from(wallet.id.to_string()),
                    balances_cell,
                    Cell::from(wallet.sent.to_string()),
                    Cell::from(wallet.received.to_string()),
                    Cell::from(wallet.failures.to_string()),
                ])
                .style(row_style)
                .height(height),
            );
        }

        let table = Table::new(
            rows,
            [
                Constraint::Length(6),
                Constraint::Min(10),
                Constraint::Length(6),
                Constraint::Length(9),
                Constraint::Length(8),
            ],
        )
        .header(Row::new(["id", "balances", "sent", "received", "fail"]))
        .block(Block::default().borders(Borders::ALL).title("Wallets"))
        .column_spacing(2);

        f.render_widget(table, body_chunks[0]);

        let logs: Vec<ListItem> = snapshot
            .logs
            .iter()
            .map(|l| ListItem::new(l.clone()))
            .collect();
        let log_block = List::new(logs).block(Block::default().borders(Borders::ALL).title("Logs"));
        f.render_widget(log_block, body_chunks[1]);

        let footer = Paragraph::new(Line::from(vec![
            Span::styled("q", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(": quit  "),
            Span::styled("p", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(": pause/resume"),
        ]))
        .block(Block::default().borders(Borders::ALL).title("Controls"));
        f.render_widget(footer, layout[2]);
    })?;

    Ok(())
}

fn wallet_color(id: usize) -> Color {
    const PALETTE: [Color; 8] = [
        Color::Cyan,
        Color::Green,
        Color::Yellow,
        Color::Magenta,
        Color::Blue,
        Color::LightRed,
        Color::LightGreen,
        Color::LightBlue,
    ];
    PALETTE[id % PALETTE.len()]
}

pub fn ui_loop(
    snapshot_rx: watch::Receiver<AppSnapshot>,
    cmd_tx: mpsc::UnboundedSender<SimCommand>,
    mut terminal: Terminal<CrosstermBackend<Stdout>>,
) -> Result<()> {
    loop {
        let snapshot = snapshot_rx.borrow().clone();
        if snapshot.shutdown {
            break;
        }

        render_ui(&mut terminal, &snapshot)?;

        if crossterm::event::poll(Duration::from_millis(100))? {
            match crossterm::event::read()? {
                Event::Key(KeyEvent {
                    code: KeyCode::Char('q'),
                    ..
                }) => {
                    let _ = cmd_tx.send(SimCommand::Quit);
                    break;
                }
                Event::Key(KeyEvent {
                    code: KeyCode::Char('p'),
                    ..
                }) => {
                    let _ = cmd_tx.send(SimCommand::TogglePause);
                }
                Event::Key(KeyEvent {
                    code: KeyCode::Char('c'),
                    modifiers: KeyModifiers::CONTROL,
                    ..
                }) => {
                    let _ = cmd_tx.send(SimCommand::Quit);
                    break;
                }
                _ => {}
            }
        }
    }
    Ok(())
}
