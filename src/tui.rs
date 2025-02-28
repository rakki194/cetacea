#![warn(clippy::all, clippy::pedantic)]

use std::{
    collections::HashMap,
    sync::{mpsc, Arc, Mutex},
    thread,
    time::Duration,
};

use ratui_lib::{
    ResponsiveGrid, TerminalApp,
    ratatui::{
        Frame,
        layout::{Constraint, Direction, Layout, Rect},
        style::{Color, Style},
        symbols,
        text::{Line, Span},
        widgets::{Axis, Chart, Dataset, GraphType},
    },
    restore_terminal, run_app, setup_terminal,
    widgets::{Card, StatusColor, StatusIndicator},
    Widget, Error as RatuiError, Event, KeyCode,
};

use crate::docker::{Container, DockerClient, Port};
use crate::utils::format_duration;

const HISTORY_SIZE: usize = 60; // 1 minute of history at 1s intervals

#[derive(Default, Clone)]
struct ResourceHistory {
    cpu_usage: Vec<(f64, f64)>,    // (timestamp, percentage)
    mem_usage: Vec<(f64, f64)>,    // (timestamp, percentage)
    gpu_usage: Vec<(f64, f64)>,    // (timestamp, percentage)
}

#[derive(PartialEq, Eq)]
enum ResourceView {
    Cpu,
    Memory,
    Gpu,
}

#[allow(dead_code)]
pub struct App {
    pub containers: Vec<Container>,
    pub should_quit: bool,
    pub refresh_rate: Duration,
    client: DockerClient,
    resource_histories: Arc<Mutex<HashMap<String, ResourceHistory>>>,
    current_view: ResourceView,
    show_graphs: bool,
    rx: mpsc::Receiver<Vec<Container>>,
}

impl App {
    pub fn new(mut containers: Vec<Container>, client: DockerClient) -> Self {
        let (tx, rx) = mpsc::channel();
        let resource_histories = Arc::new(Mutex::new(
            containers
                .iter()
                .map(|c| (c.id.clone(), ResourceHistory::default()))
                .collect::<HashMap<String, ResourceHistory>>(),
        ));

        // Sort containers: running first, then by name
        containers.sort_by(|a, b| {
            let a_running = a.state == "running";
            let b_running = b.state == "running";
            match (a_running, b_running) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => {
                    let a_name = a.names.first().map_or("", |s| s.as_str());
                    let b_name = b.names.first().map_or("", |s| s.as_str());
                    a_name.cmp(b_name)
                },
            }
        });

        // Spawn container update thread
        let update_client = client.clone();
        thread::spawn(move || {
            loop {
                thread::sleep(Duration::from_secs(1));
                if let Ok(mut containers) = update_client.list_containers_blocking() {
                    containers.sort_by(|a, b| {
                        let a_running = a.state == "running";
                        let b_running = b.state == "running";
                        match (a_running, b_running) {
                            (true, false) => std::cmp::Ordering::Less,
                            (false, true) => std::cmp::Ordering::Greater,
                            _ => {
                                let a_name = a.names.first().map_or("", |s| s.as_str());
                                let b_name = b.names.first().map_or("", |s| s.as_str());
                                a_name.cmp(b_name)
                            },
                        }
                    });
                    if tx.send(containers).is_err() {
                        break;
                    }
                }
            }
        });

        // Spawn stats update thread
        let stats_client = client.clone();
        let stats_histories = Arc::clone(&resource_histories);
        thread::spawn(move || {
            loop {
                thread::sleep(Duration::from_secs(1));
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs_f64();

                if let Ok(mut histories) = stats_histories.lock() {
                    for (id, history) in histories.iter_mut() {
                        if let Ok(stats) = stats_client.get_container_stats_blocking(id) {
                            // Update CPU usage
                            if let (Some(system_cpu), Some(online_cpus)) = (
                                stats.cpu_stats.system_cpu_usage,
                                stats.cpu_stats.online_cpus,
                            ) {
                                let cpu_delta = stats.cpu_stats.cpu_usage.total_usage as f64
                                    - stats.precpu_stats.cpu_usage.total_usage as f64;
                                let system_delta = system_cpu as f64
                                    - stats.precpu_stats.system_cpu_usage.unwrap_or(0) as f64;
                                if system_delta > 0.0 {
                                    let cpu_percent = (cpu_delta / system_delta) * 100.0 * online_cpus as f64;
                                    history.cpu_usage.push((now, cpu_percent));
                                    if history.cpu_usage.len() > HISTORY_SIZE {
                                        history.cpu_usage.remove(0);
                                    }
                                }
                            }

                            // Update memory usage
                            if let (Some(usage), Some(limit)) = (stats.memory_stats.usage, stats.memory_stats.limit) {
                                let mem_percent = (usage as f64 / limit as f64) * 100.0;
                                history.mem_usage.push((now, mem_percent));
                                if history.mem_usage.len() > HISTORY_SIZE {
                                    history.mem_usage.remove(0);
                                }
                            }

                            // Update GPU usage if available
                            if let Some(gpu_stats) = stats.gpu_stats {
                                if !gpu_stats.devices.is_empty() {
                                    let gpu = &gpu_stats.devices[0]; // Use first GPU for now
                                    let gpu_percent = gpu.utilization as f64;
                                    history.gpu_usage.push((now, gpu_percent));
                                    if history.gpu_usage.len() > HISTORY_SIZE {
                                        history.gpu_usage.remove(0);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        });

        Self {
            containers,
            client,
            should_quit: false,
            refresh_rate: Duration::from_millis(250),
            resource_histories,
            current_view: ResourceView::Cpu,
            show_graphs: true,
            rx,
        }
    }

    fn update(&mut self) {
        // Try to receive container updates
        while let Ok(new_containers) = self.rx.try_recv() {
            self.containers = new_containers;
            // Update resource histories map with new containers
            if let Ok(mut histories) = self.resource_histories.lock() {
                let mut new_histories = HashMap::new();
                for container in &self.containers {
                    new_histories.insert(
                        container.id.clone(),
                        histories
                            .remove(&container.id)
                            .unwrap_or_default(),
                    );
                }
                *histories = new_histories;
            }
        }
    }

    fn render_container(f: &mut Frame, container: &Container, area: Rect, history: &ResourceHistory, view: &ResourceView, show_graphs: bool) {
        let status_color = match container.state.as_str() {
            "running" => {
                if let Some(health) = &container.health {
                    match health.status.as_str() {
                        "healthy" => StatusColor::Success,
                        "unhealthy" => StatusColor::Error,
                        _ => StatusColor::Warning,
                    }
                } else {
                    StatusColor::Success
                }
            }
            _ => StatusColor::Error,
        };

        // Use minimal view for very narrow widths
        if area.width < 30 {
            let name = container.names.first().map_or("", |s| s.as_str());
            StatusIndicator::new(status_color)
                .label(name)
                .render(area, f.buffer_mut());
            return;
        }

        // Split area into info and graph sections if showing graphs and container is running
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                if show_graphs && container.state == "running" {
                    vec![
                        Constraint::Min(6),     // Container info gets remaining space
                        Constraint::Length(15), // Fixed height for graph
                    ]
                } else {
                    vec![Constraint::Min(0)]  // Use full area for container info
                }
            )
            .split(area);

        // Render container info
        let names = if container.names.is_empty() {
            String::new()
        } else {
            container.names.join(", ")
        };
        let title = format!("{} ({})", names, &container.id[..12]);
        let ports_str = format_ports(&container.ports);
        let content = vec![
            Line::from(vec![Span::raw(&container.image)]),
            Line::from(vec![Span::raw(&container.command)]),
            Line::from(vec![Span::raw(format_duration(container.created))]),
            Line::from(vec![Span::raw(&container.status)]),
            Line::from(vec![Span::raw(format!("Ports: {ports_str}"))]),
        ];

        Card::new()
            .title(&title)
            .content(content)
            .border_style(Style::default().fg(status_color.into()))
            .render(chunks[0], f.buffer_mut());

        // Only render graph if we're showing graphs and have a second chunk
        if show_graphs && container.state == "running" && chunks.len() > 1 {
            let datasets = match view {
                ResourceView::Cpu => vec![Dataset::default()
                    .name("CPU %")
                    .marker(symbols::Marker::Braille)
                    .graph_type(GraphType::Line)
                    .style(Style::default().fg(Color::Cyan))
                    .data(&history.cpu_usage)],
                ResourceView::Memory => vec![Dataset::default()
                    .name("Memory %")
                    .marker(symbols::Marker::Braille)
                    .graph_type(GraphType::Line)
                    .style(Style::default().fg(Color::Green))
                    .data(&history.mem_usage)],
                ResourceView::Gpu => vec![Dataset::default()
                    .name("GPU %")
                    .marker(symbols::Marker::Braille)
                    .graph_type(GraphType::Line)
                    .style(Style::default().fg(Color::Yellow))
                    .data(&history.gpu_usage)],
            };

            let chart = Chart::new(datasets)
                .block(
                    ratui_lib::ratatui::widgets::Block::default()
                        .borders(ratui_lib::ratatui::widgets::Borders::ALL)
                        .title(match view {
                            ResourceView::Cpu => "CPU Usage",
                            ResourceView::Memory => "Memory Usage",
                            ResourceView::Gpu => "GPU Usage",
                        }),
                )
                .x_axis(
                    Axis::default()
                        .style(Style::default().fg(Color::Gray))
                        .bounds([
                            history.cpu_usage.first().map(|p| p.0).unwrap_or_default(),
                            history.cpu_usage.last().map(|p| p.0).unwrap_or_default(),
                        ]),
                )
                .y_axis(
                    Axis::default()
                        .style(Style::default().fg(Color::Gray))
                        .bounds([0.0, 100.0])
                        .labels(vec!["0%", "25%", "50%", "75%", "100%"]
                            .into_iter()
                            .map(Span::raw)
                            .collect::<Vec<_>>()),
                );

            f.render_widget(chart, chunks[1]);
        }
    }

    pub fn run_with_options(mut self, refresh_rate: Duration) -> Result<(), RatuiError> {
        self.refresh_rate = refresh_rate;
        let mut terminal = setup_terminal()?;
        run_app(&mut terminal, self)?;
        restore_terminal()?;
        Ok(())
    }
}

impl TerminalApp for App {
    fn ui(&self, f: &mut Frame) {
        let grid = ResponsiveGrid::new();
        let cells = grid.split(f.area(), self.containers.len());

        for (container, area) in self.containers.iter().zip(cells) {
            if let Ok(histories) = self.resource_histories.lock() {
                if let Some(history) = histories.get(&container.id) {
                    Self::render_container(f, container, area, history, &self.current_view, self.show_graphs);
                }
            }
        }
    }

    fn handle_event(&mut self, event: Event) -> anyhow::Result<bool> {
        // Update container list and resource histories
        self.update();

        if let Event::Key(key) = event {
            match key.code {
                KeyCode::Char('q') => {
                    self.should_quit = true;
                    return Ok(true);
                }
                KeyCode::Char('g') => {
                    self.show_graphs = !self.show_graphs;
                }
                KeyCode::Left => {
                    self.current_view = match self.current_view {
                        ResourceView::Cpu => ResourceView::Gpu,
                        ResourceView::Memory => ResourceView::Cpu,
                        ResourceView::Gpu => ResourceView::Memory,
                    };
                }
                KeyCode::Right => {
                    self.current_view = match self.current_view {
                        ResourceView::Cpu => ResourceView::Memory,
                        ResourceView::Memory => ResourceView::Gpu,
                        ResourceView::Gpu => ResourceView::Cpu,
                    };
                }
                _ => {}
            }
        }

        Ok(false)
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

            if let Some(ip) = &p.ip {
                if !ip.is_empty() {
                    parts.push(ip.to_string());
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
                parts[0].clone()
            };

            format!("{}/{}", addr_port, p.protocol.to_lowercase())
        })
        .collect::<Vec<_>>()
        .join(", ")
}
