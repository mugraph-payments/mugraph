use std::{
    collections::VecDeque,
    fmt::Write,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use crossterm::{
    ExecutableCommand,
    event::{self, Event, KeyCode},
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Style},
    symbols,
    text::{Line, Span},
    widgets::{Block, Borders, Chart, Dataset, GraphType, List, ListItem, Paragraph},
};
use time::{OffsetDateTime, format_description::FormatItem, macros::format_description};
use tokio::sync::mpsc;
use tracing::Subscriber;
use tracing_subscriber::{
    fmt::{FmtContext, FormatEvent, FormatFields, format::Writer},
    registry::LookupSpan,
};

const TIME_FORMAT: &[FormatItem<'static>] = format_description!("[hour]:[minute]:[second]");

pub struct DashboardFormatter {
    logs: Arc<Mutex<VecDeque<String>>>,
}

impl DashboardFormatter {
    pub fn new() -> (Self, Arc<Mutex<VecDeque<String>>>) {
        let logs = Arc::new(Mutex::new(VecDeque::with_capacity(1000)));
        (Self { logs: logs.clone() }, logs)
    }
}

impl<S, N> FormatEvent<S, N> for DashboardFormatter
where
    S: Subscriber + for<'a> LookupSpan<'a>,
    N: for<'a> FormatFields<'a> + 'static,
{
    fn format_event(
        &self,
        ctx: &FmtContext<'_, S, N>,
        mut writer: Writer<'_>,
        event: &tracing::Event<'_>,
    ) -> std::fmt::Result {
        // Create a String to store our formatted log
        let mut log_line = String::new();

        // Write timestamp
        let now = OffsetDateTime::now_local().unwrap_or_else(|_| OffsetDateTime::now_utc());
        let formatted_time = now
            .format(TIME_FORMAT)
            .unwrap_or_else(|_| "--:--:--".to_string());
        write!(log_line, "[{}] ", formatted_time)?;

        // Write level
        write!(log_line, "{:>5} ", event.metadata().level())?;

        ctx.field_format().format_fields(writer.by_ref(), event)?;

        log_line.push('\n');

        // Store in logs
        let mut logs = self.logs.lock().unwrap();
        logs.push_front(log_line);

        if logs.len() > 1000 {
            logs.pop_back();
        }

        Ok(())
    }
}

#[derive(Debug)]
pub enum DashboardEvent {
    Metrics {
        elapsed: f64,
        tps: u64,
        p50: u64,
        p90: u64,
        p99: u64,
    },
}

pub struct Dashboard {
    terminal: Terminal<CrosstermBackend<std::io::Stdout>>,
    rx: mpsc::UnboundedReceiver<DashboardEvent>,
    logs: Arc<Mutex<VecDeque<String>>>,
    tps_history: VecDeque<(f64, f64)>,
    latency_history: VecDeque<(f64, f64)>,
    window_size: usize,
    last_update: Instant,
    current_p50: u64,
    current_p90: u64,
    current_p99: u64,
}

impl Dashboard {
    pub fn new(logs: Arc<Mutex<VecDeque<String>>>) -> (Self, mpsc::UnboundedSender<DashboardEvent>) {
        let (tx, rx) = mpsc::unbounded_channel();

        enable_raw_mode().expect("Failed to enable raw mode");
        let mut stdout = std::io::stdout();
        stdout
            .execute(EnterAlternateScreen)
            .expect("Failed to enter alternate screen");
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend).expect("Failed to create terminal");

        (
            Self {
                terminal,
                rx,
                logs,
                tps_history: VecDeque::with_capacity(100),
                latency_history: VecDeque::with_capacity(100),
                window_size: 100,
                last_update: Instant::now(),
                current_p50: 0,
                current_p90: 0,
                current_p99: 0,
            },
            tx,
        )
    }

    pub fn run(&mut self) -> std::io::Result<()> {
        loop {
            // Handle any pending events
            while let Ok(event) = self.rx.try_recv() {
                let DashboardEvent::Metrics {
                    elapsed,
                    tps,
                    p50,
                    p90,
                    p99,
                } = event;
                self.current_p50 = p50;
                self.current_p90 = p90;
                self.current_p99 = p99;
                // Only update every 100ms to avoid too frequent updates
                if self.last_update.elapsed() >= Duration::from_millis(100) {
                    self.tps_history.push_back((elapsed, tps as f64));
                    if self.tps_history.len() > self.window_size {
                        self.tps_history.pop_front();
                    }

                    self.latency_history.push_back((elapsed, p99 as f64));
                    if self.latency_history.len() > self.window_size {
                        self.latency_history.pop_front();
                    }

                    self.last_update = Instant::now();
                }
            }

            // Draw the UI
            self.terminal.draw(|f| {
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Percentage(30), // TPS graph
                        Constraint::Percentage(30), // Latency graph
                        Constraint::Percentage(40), // Logs + metrics
                    ])
                    .split(f.area());

                // Split the bottom area into logs and metrics
                let log_chunks = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                    .split(chunks[2]);

                // Draw TPS Graph
                self.tps_history.make_contiguous();
                let tps_dataset = Dataset::default()
                    .name("TPS")
                    .graph_type(GraphType::Line)
                    .style(Style::default().fg(Color::Cyan))
                    .marker(symbols::Marker::Braille)
                    .data(self.tps_history.as_slices().0);

                let tps_chart = Chart::new(vec![tps_dataset])
                    .block(
                        Block::default()
                            .title("Transactions Per Second")
                            .borders(Borders::ALL),
                    )
                    .x_axis(
                        ratatui::widgets::Axis::default()
                            .style(Style::default().fg(Color::Gray))
                            .bounds([
                                self.tps_history.front().map(|(x, _)| *x).unwrap_or(0.0),
                                self.tps_history.back().map(|(x, _)| *x).unwrap_or(0.0),
                            ]),
                    )
                    .y_axis(
                        ratatui::widgets::Axis::default()
                            .style(Style::default().fg(Color::Gray))
                            .bounds([
                                0.0,
                                self.tps_history
                                    .iter()
                                    .map(|p| p.1)
                                    .fold(0.0, f64::max)
                                    .max(1.0),
                            ]),
                    );

                // Draw Latency Graph
                self.latency_history.make_contiguous();
                let latency_dataset = Dataset::default()
                    .name("P99 Latency")
                    .graph_type(GraphType::Line)
                    .style(Style::default().fg(Color::Yellow))
                    .marker(symbols::Marker::Braille)
                    .data(self.latency_history.as_slices().0);

                let latency_chart = Chart::new(vec![latency_dataset])
                    .block(Block::default().title("Latency (ms)").borders(Borders::ALL))
                    .x_axis(
                        ratatui::widgets::Axis::default()
                            .style(Style::default().fg(Color::Gray))
                            .bounds([
                                self.latency_history.front().map(|(x, _)| *x).unwrap_or(0.0),
                                self.latency_history.back().map(|(x, _)| *x).unwrap_or(0.0),
                            ]),
                    )
                    .y_axis(
                        ratatui::widgets::Axis::default()
                            .style(Style::default().fg(Color::Gray))
                            .bounds([
                                0.0,
                                self.latency_history
                                    .iter()
                                    .map(|p| p.1)
                                    .fold(0.0, f64::max)
                                    .max(100.0),
                            ]),
                    );

                f.render_widget(tps_chart, chunks[0]);
                f.render_widget(latency_chart, chunks[1]);

                // Draw Logs (left side)
                let log_items: Vec<ListItem> = self
                    .logs
                    .lock()
                    .unwrap()
                    .clone()
                    .into_iter()
                    .rev() // Show latest logs first
                    .take(log_chunks[0].height as usize - 2) // -2 for borders
                    .map(|s| ListItem::new(s.clone()))
                    .collect();

                let logs = List::new(log_items)
                    .block(Block::default().title("Logs").borders(Borders::ALL))
                    .style(Style::default().fg(Color::White));

                // Draw Metrics (right side)
                let current_tps = self.tps_history.back().map(|(_, tps)| *tps).unwrap_or(0.0);
                let avg_tps = self.tps_history.iter().map(|(_, t)| t).sum::<f64>()
                    / self.tps_history.len().max(1) as f64;

                let metrics_text = vec![
                    Line::from(Span::raw("")),
                    Line::from(vec![
                        Span::styled("Current TPS: ", Style::default().fg(Color::Cyan)),
                        Span::styled(
                            format!("{:.2}", current_tps),
                            Style::default().fg(Color::White),
                        ),
                    ]),
                    Line::from(vec![
                        Span::styled("Average TPS: ", Style::default().fg(Color::Cyan)),
                        Span::styled(format!("{:.2}", avg_tps), Style::default().fg(Color::White)),
                    ]),
                    Line::from(Span::raw("")),
                    Line::from(vec![
                        Span::styled("P50 Latency: ", Style::default().fg(Color::Yellow)),
                        Span::styled(
                            format!("{}ms", self.current_p50),
                            Style::default().fg(Color::White),
                        ),
                    ]),
                    Line::from(vec![
                        Span::styled("P90 Latency: ", Style::default().fg(Color::Yellow)),
                        Span::styled(
                            format!("{}ms", self.current_p90),
                            Style::default().fg(Color::White),
                        ),
                    ]),
                    Line::from(vec![
                        Span::styled("P99 Latency: ", Style::default().fg(Color::Yellow)),
                        Span::styled(
                            format!("{}ms", self.current_p99),
                            Style::default().fg(Color::White),
                        ),
                    ]),
                ];

                let metrics = Paragraph::new(metrics_text)
                    .block(Block::default().title("Metrics").borders(Borders::ALL))
                    .alignment(Alignment::Left);

                f.render_widget(logs, log_chunks[0]);
                f.render_widget(metrics, log_chunks[1]);
            })?;

            // Check for quit
            if event::poll(std::time::Duration::from_millis(100))?
                && let Event::Key(key) = event::read()?
                    && key.code == KeyCode::Char('q') {
                        break;
                    }
        }

        Ok(())
    }
}

impl Drop for Dashboard {
    fn drop(&mut self) {
        disable_raw_mode().ok();
        self.terminal
            .backend_mut()
            .execute(LeaveAlternateScreen)
            .ok();
    }
}
