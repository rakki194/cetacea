#![warn(clippy::all, clippy::pedantic)]
#![allow(clippy::cast_possible_truncation, clippy::cast_sign_loss, clippy::cast_precision_loss)]

use std::io::{self, stdout, Stdout};
use std::sync::mpsc::{self, Receiver};
use std::thread;
use std::time::Duration;

use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame, Terminal,
};

use crate::docker::{Container, DockerClient, Port};
use crate::format_duration;

pub struct App {
    pub containers: Vec<Container>,
    #[allow(dead_code)]  // Used through cloning in the update thread
    client: DockerClient,
    update_rx: Receiver<Vec<Container>>,
}

impl App {
    pub fn new(mut containers: Vec<Container>, client: DockerClient) -> Self {
        let (tx, rx) = mpsc::channel();
        
        // Sort containers: running first, then by name
        containers.sort_by(|a, b| {
            // First sort by state (running before stopped)
            let a_running = a.state == "running";
            let b_running = b.state == "running";
            match (a_running, b_running) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => a.names[0].cmp(&b.names[0]), // If states are equal, sort by name
            }
        });

        // Spawn container update thread
        let update_client = client.clone();
        thread::spawn(move || {
            loop {
                thread::sleep(Duration::from_secs(1));
                if let Ok(mut containers) = update_client.list_containers_blocking() {
                    // Sort containers: running first, then by name
                    containers.sort_by(|a, b| {
                        // First sort by state (running before stopped)
                        let a_running = a.state == "running";
                        let b_running = b.state == "running";
                        match (a_running, b_running) {
                            (true, false) => std::cmp::Ordering::Less,
                            (false, true) => std::cmp::Ordering::Greater,
                            _ => a.names[0].cmp(&b.names[0]), // If states are equal, sort by name
                        }
                    });
                    if tx.send(containers).is_err() {
                        break;
                    }
                }
            }
        });

        Self {
            containers,
            client,
            update_rx: rx,
        }
    }

    pub fn run(&mut self) -> io::Result<()> {
        enable_raw_mode()?;
        let mut stdout = stdout();
        execute!(stdout, EnterAlternateScreen)?;

        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        let res = self.run_app(&mut terminal);

        disable_raw_mode()?;
        execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
        terminal.show_cursor()?;

        res
    }

    fn run_app(&mut self, terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> io::Result<()> {
        loop {
            // Check for container updates
            if let Ok(new_containers) = self.update_rx.try_recv() {
                self.containers = new_containers;
            }

            terminal.draw(|f| self.ui(f))?;

            // Poll for user input with a timeout
            if event::poll(Duration::from_millis(100))? {
                if let Event::Key(key) = event::read()? {
                    if key.code == KeyCode::Char('q') {
                        return Ok(());
                    }
                }
            }
        }
    }

    fn ui(&self, f: &mut Frame) {
        let size = f.area();
        let container_count = self.containers.len();
        let optimal_columns = match size.width {
            0..=50 => 1,
            51..=100 => 2,
            101..=150 => 3,
            _ => 4,
        };

        let rows = (container_count as f32 / optimal_columns as f32).ceil() as usize;
        let mut constraints = vec![];
        for _ in 0..rows {
            constraints.push(Constraint::Ratio(1, u32::try_from(rows).unwrap_or(1)));
        }

        let vertical_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(constraints)
            .split(size);

        let mut row_constraints = vec![];
        for _ in 0..optimal_columns {
            row_constraints.push(Constraint::Ratio(1, u32::try_from(optimal_columns).unwrap_or(1)));
        }

        for (row_idx, row) in vertical_chunks.iter().enumerate() {
            let horizontal_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints(row_constraints.clone())
                .split(*row);

            for col_idx in 0..optimal_columns {
                let container_idx = row_idx * optimal_columns + col_idx;
                if container_idx < container_count {
                    if let Some(container) = self.containers.get(container_idx) {
                        Self::render_container(f, container, horizontal_chunks[col_idx]);
                    }
                }
            }
        }
    }

    fn render_container(f: &mut Frame, container: &Container, area: Rect) {
        let status_color = match container.state.as_str() {
            "running" => {
                if let Some(health) = &container.health {
                    match health.status.as_str() {
                        "healthy" => Color::Green,
                        "unhealthy" => Color::Red,
                        _ => Color::Yellow,
                    }
                } else {
                    Color::Green
                }
            }
            _ => Color::Red,
        };

        // Use minimal view for very narrow widths
        if area.width < 30 {
            let status_dot = match container.state.as_str() {
                "running" => "●",
                _ => "○",
            };
            let content = Text::from(vec![Line::from(vec![
                Span::styled(status_dot, Style::default().fg(status_color)),
                Span::raw(" "),
                Span::raw(&container.names[0]),
            ])]);
            let paragraph = Paragraph::new(content);
            f.render_widget(paragraph, area);
            return;
        }

        let title = format!("{} ({})", container.names.join(", "), &container.id[..12]);
        let block = Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(status_color));

        let ports_str = format_ports(&container.ports);
        let content = Text::from(vec![
            Line::from(vec![Span::raw(&container.image)]),
            Line::from(vec![Span::raw(&container.command)]),
            Line::from(vec![Span::raw(format_duration(container.created))]),
            Line::from(vec![Span::raw(&container.status)]),
            Line::from(vec![Span::raw(format!("Ports: {ports_str}"))]),
        ]);

        let paragraph = Paragraph::new(content)
            .block(block)
            .wrap(Wrap { trim: true });

        f.render_widget(paragraph, area);
    }
}

pub fn format_ports(ports: &[Port]) -> String {
    if ports.is_empty() {
        return "None".to_string();
    }

    ports
        .iter()
        .map(|p| {
            let mut parts = Vec::new();
            
            if let Some(ref ip) = p.ip {
                if !ip.is_empty() {
                    parts.push(ip.clone());
                }
            }
            
            if let Some(external) = p.external {
                parts.push(external.to_string());
            }
            
            parts.push(p.internal.to_string());
            
            // Format as address:port/protocol
            let addr_port = if parts.len() > 1 {
                parts.join(":")
            } else {
                parts[0].to_string()
            };
            
            format!("{}/{}", addr_port, p.protocol.to_lowercase())
        })
        .collect::<Vec<_>>()
        .join(", ")
} 