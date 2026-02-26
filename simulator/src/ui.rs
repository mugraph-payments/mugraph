use std::{io::Stdout, time::Duration};

use color_eyre::eyre::Result;
use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Row, Table},
};
use tokio::sync::{mpsc, watch};

use crate::types::{AppSnapshot, SimCommand};

#[derive(Clone, Copy, PartialEq)]
enum ViewState {
    Dashboard,
    Wallets,
}

fn render_ui(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    snapshot: &AppSnapshot,
    view: ViewState,
) -> Result<()> {
    terminal.draw(|f| {
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([
                Constraint::Length(3), // Conservation banner
                Constraint::Length(4), // Health metrics
                Constraint::Min(5),    // Main body (assets or wallets)
                Constraint::Length(6), // Log tail
                Constraint::Length(1), // Controls
            ])
            .split(f.area());

        // === Conservation banner ===
        let node_label = if snapshot.node_count == 1 {
            "1 node".to_string()
        } else {
            format!("{} nodes", snapshot.node_count)
        };

        let conservation_line = Line::from(vec![
            Span::styled(
                " CONSERVED ",
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("  "),
            Span::styled(
                format!("{}", snapshot.conservation_checks),
                Style::default().fg(Color::Green),
            ),
            Span::raw(" checks   "),
            Span::styled(node_label, Style::default().fg(Color::Cyan)),
        ]);

        let conservation = Paragraph::new(conservation_line)
            .block(Block::default().borders(Borders::ALL).title("Conservation"));
        f.render_widget(conservation, layout[0]);

        // === Health metrics ===
        let paused = snapshot.paused;
        let health = Paragraph::new(vec![
            Line::from(vec![
                Span::raw(" tx/s: "),
                Span::styled(
                    format!("{:.1}", snapshot.tx_per_sec),
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw("   ok: "),
                Span::styled(
                    format!("{:.1}%", snapshot.success_rate),
                    Style::default().fg(if snapshot.success_rate >= 95.0 {
                        Color::Green
                    } else if snapshot.success_rate >= 80.0 {
                        Color::Yellow
                    } else {
                        Color::Red
                    }),
                ),
                Span::raw("   inflight: "),
                Span::styled(
                    format!("{}/{}", snapshot.inflight, snapshot.max_inflight),
                    Style::default().fg(Color::Yellow),
                ),
                Span::raw("   paused: "),
                Span::styled(
                    format!("{paused}"),
                    Style::default().fg(if paused { Color::Yellow } else { Color::Green }),
                ),
            ]),
            Line::from(vec![
                Span::raw(" total: "),
                Span::styled(
                    format!("{}", snapshot.total_sent),
                    Style::default().fg(Color::Magenta),
                ),
                Span::raw(" sent  "),
                Span::styled(
                    format!("{}", snapshot.total_ok),
                    Style::default().fg(Color::Green),
                ),
                Span::raw(" ok  "),
                Span::styled(
                    format!("{}", snapshot.total_err),
                    Style::default().fg(Color::Red),
                ),
                Span::raw(" err"),
                if let Some(ref last_err) = snapshot.last_failure {
                    Span::styled(
                        format!("   last: {last_err}"),
                        Style::default().fg(Color::Red),
                    )
                } else {
                    Span::raw("")
                },
            ]),
        ])
        .block(Block::default().borders(Borders::ALL).title("Health"));
        f.render_widget(health, layout[1]);

        // === Main body ===
        match view {
            ViewState::Dashboard => {
                let wallet_count = snapshot.wallets.len();
                let rows: Vec<Row> = snapshot
                    .asset_summaries
                    .iter()
                    .map(|a| {
                        let short_policy = &a.policy_id_hex[..8];
                        Row::new([
                            a.name.to_string(),
                            short_policy.to_string(),
                            format!("{}", a.total_supply),
                            format!("{}", a.total_notes),
                            format!("{}/{}", a.wallets_holding, wallet_count),
                        ])
                    })
                    .collect();

                let table = Table::new(
                    rows,
                    [
                        Constraint::Length(18),
                        Constraint::Length(10),
                        Constraint::Length(12),
                        Constraint::Length(8),
                        Constraint::Length(10),
                    ],
                )
                .header(
                    Row::new(["name", "policy", "supply", "notes", "wallets"])
                        .style(Style::default().add_modifier(Modifier::BOLD)),
                )
                .block(Block::default().borders(Borders::ALL).title("Assets"))
                .column_spacing(2);

                f.render_widget(table, layout[2]);
            }
            ViewState::Wallets => {
                let rows: Vec<Row> = snapshot
                    .wallets
                    .iter()
                    .enumerate()
                    .map(|(i, w)| {
                        let total_balance: u64 = w.balances.iter().map(|b| b.balance).sum();
                        let total_notes: usize = w.balances.iter().map(|b| b.notes).sum();
                        Row::new([
                            format!("{}", w.id),
                            format!("N{}", w.home_node),
                            format!("{total_balance}"),
                            format!("{total_notes}"),
                            format!("{}", w.sent),
                            format!("{}", w.received),
                            format!("{}", w.failures),
                        ])
                        .style(Style::default().fg(wallet_color(i)))
                    })
                    .collect();

                let table = Table::new(
                    rows,
                    [
                        Constraint::Length(4),
                        Constraint::Length(6),
                        Constraint::Length(12),
                        Constraint::Length(8),
                        Constraint::Length(8),
                        Constraint::Length(8),
                        Constraint::Length(8),
                    ],
                )
                .header(
                    Row::new(["id", "home", "balance", "notes", "sent", "recv", "fail"])
                        .style(Style::default().add_modifier(Modifier::BOLD)),
                )
                .block(Block::default().borders(Borders::ALL).title("Wallets"))
                .column_spacing(2);

                f.render_widget(table, layout[2]);
            }
        }

        // === Log tail ===
        let logs: Vec<ListItem> = snapshot
            .logs
            .iter()
            .take(4)
            .map(|l| ListItem::new(l.clone()))
            .collect();
        let log_block = List::new(logs).block(Block::default().borders(Borders::ALL).title("Log"));
        f.render_widget(log_block, layout[3]);

        // === Controls (single line, no border) ===
        let toggle_label = match view {
            ViewState::Dashboard => "wallets",
            ViewState::Wallets => "dashboard",
        };
        let controls = Paragraph::new(Line::from(vec![
            Span::raw(" "),
            Span::styled("q", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(": quit  "),
            Span::styled("p", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(": pause  "),
            Span::styled("w", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(format!(": {toggle_label}")),
        ]));
        f.render_widget(controls, layout[4]);
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
    let mut view = ViewState::Dashboard;

    loop {
        let snapshot = snapshot_rx.borrow().clone();
        if snapshot.shutdown {
            break;
        }

        render_ui(&mut terminal, &snapshot, view)?;

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
                    code: KeyCode::Char('w'),
                    ..
                }) => {
                    view = match view {
                        ViewState::Dashboard => ViewState::Wallets,
                        ViewState::Wallets => ViewState::Dashboard,
                    };
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
