use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap},
    Frame, Terminal,
    text::{Line, Span, Text},
};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap, io, path::PathBuf, process::Command, time::Duration,
};

// --- DATA STRUCTURES ---

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OpFile {
    pub operations: Vec<OpEntry>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OpEntry {
    pub id: String,
    pub status: String,
    pub target_file: String,
    pub gcs_uri: Option<String>,
    pub created_at: Option<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

// --- APP STATE ---

struct App {
    operations: Vec<OpEntry>,
    state: ListState,
    log_buffer: Vec<String>,
    is_loading: bool,
}

impl App {
    fn new() -> App {
        App {
            operations: Vec::new(),
            state: ListState::default(),
            log_buffer: vec!["System Initialized. Idle.".to_string()],
            is_loading: false,
        }
    }

    fn next(&mut self) {
        if self.operations.is_empty() {
            return;
        }
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.operations.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    fn previous(&mut self) {
        if self.operations.is_empty() {
            return;
        }
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.operations.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    fn log(&mut self, msg: &str) {
        self.log_buffer.push(format!(
            "[{}] {}",
            chrono::Local::now().format("%H:%M:%S"),
            msg
        ));
        if self.log_buffer.len() > 50 {
            self.log_buffer.remove(0);
        }
    }

    // Refresh data from disk and API
    fn refresh(&mut self) {
        self.is_loading = true;
        // 1. Find File
        let paths = vec![
            PathBuf::from(".sentinel/operations.json"),
            PathBuf::from("_Agentes/.sentinel/operations.json"),
            PathBuf::from("../_Agentes/.sentinel/operations.json"),
        ];

        let mut ops_path = PathBuf::new();
        for p in paths {
            if p.exists() {
                ops_path = p;
                break;
            }
        }

        if !ops_path.exists() {
            self.operations.clear();
            self.log("No operations.json found.");
            self.is_loading = false;
            return;
        }

        // 2. Read File
        match std::fs::read_to_string(&ops_path) {
            Ok(content) => {
                let store: Result<OpFile, _> = serde_json::from_str(&content);
                if let Ok(s) = store {
                    self.operations = s.operations;
                    self.operations.reverse(); // Newest first usually better for monitoring
                }
            }
            Err(e) => self.log(&format!("Read Error: {}", e)),
        }
        self.is_loading = false;
    }

    fn check_selected(&mut self) {
        if let Some(i) = self.state.selected() {
            if let Some(op) = self.operations.get(i).cloned() {
                if op.status != "Running" {
                    self.log(&format!(
                        "Skipping check for {} (Status: {})",
                        op.target_file, op.status
                    ));
                    return;
                }

                self.log(&format!("Checking Google API for {}...", op.target_file));

                // Spawn thread to avoid blocking UI? For now, blocking is safer preventing race conditions in this simple TUI
                // But to make UI responsive we should really channel this.
                // We'll do a quick blocking check for now as user wants ACCURACY over 60fps animations.

                // ... (Logic from recover command) ...
                let token_out = Command::new("gcloud")
                    .args(&["auth", "print-access-token"])
                    .output();
                if let Ok(t) = token_out {
                    let token = String::from_utf8_lossy(&t.stdout).trim().to_string();

                    let parts: Vec<&str> = op.id.split('/').collect();
                    let uuid = parts.last().unwrap_or(&"unknown");
                    let mut proj = Command::new("gcloud")
                        .args(&["config", "get-value", "project"])
                        .output()
                        .ok()
                        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
                        .unwrap_or_else(|| "unknown".to_string());
                    let mut loc = "us-central1";
                    if parts.len() > 4 {
                        proj = parts[1].to_string();
                        loc = parts[3];
                    }

                    let url = format!("https://{}-aiplatform.googleapis.com/v1beta1/projects/{}/locations/{}/operations/{}", loc, proj, loc, uuid);

                    let out = Command::new("curl")
                        .arg("-f")
                        .arg("-s")
                        .arg("-S")
                        .arg("-H")
                        .arg(format!("Authorization: Bearer {}", token))
                        .arg(url)
                        .output();

                    if let Ok(res) = out {
                        if res.status.success() {
                            let json: serde_json::Value =
                                serde_json::from_str(&String::from_utf8_lossy(&res.stdout))
                                    .unwrap_or(serde_json::json!({}));
                            if let Some(done) = json.get("done").and_then(|b| b.as_bool()) {
                                if done {
                                    self.log("✅ Operation DONE!");
                                    self.update_status(i, "Completed".to_string());
                                } else {
                                    self.log("⏳ Still Running...");
                                }
                            }
                            if let Some(err) = json.get("error") {
                                let msg = err["message"].as_str().unwrap_or("Unknown error");
                                self.log(&format!("❌ Failed: {}", msg));
                                self.update_status(i, "Failed".to_string());
                            }
                        } else {
                            let err = String::from_utf8_lossy(&res.stderr);
                            if err.contains("404") {
                                self.log("🗑️ 404 Not Found (Lost)");
                                self.update_status(i, "Lost".to_string());
                            } else {
                                self.log(&format!("⚠️ API Error: {}", err));
                            }
                        }
                    }
                }
            }
        }
    }

    fn update_status(&mut self, idx: usize, status: String) {
        if let Some(op) = self.operations.get_mut(idx) {
            op.status = status;
        }
        // Save to disk
        self.save();
    }

    fn save(&self) {
        let paths = vec![
            PathBuf::from(".sentinel/operations.json"),
            PathBuf::from("_Agentes/.sentinel/operations.json"),
        ];
        for p in paths {
            if p.exists() {
                let store = OpFile {
                    operations: self.operations.clone().into_iter().rev().collect(),
                }; // Reverse back for storage order if needed, or just store as is
                let _ = std::fs::write(p, serde_json::to_string_pretty(&store).unwrap());
                break;
            }
        }
    }
}

// --- UI RUNNER ---

pub fn run_tui() -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let res = run_app(&mut terminal, App::new());

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err);
    }

    Ok(())
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> io::Result<()> 
where std::io::Error: From<<B as Backend>::Error> 
{
    // Initial Load
    app.refresh();
    app.state.select(Some(0));

    loop {
        terminal.draw(|f| ui(f, &mut app))?;

        if event::poll(Duration::from_millis(250))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => return Ok(()),
                    KeyCode::Down => app.next(),
                    KeyCode::Up => app.previous(),
                    KeyCode::Char('r') => app.refresh(),
                    KeyCode::Enter => app.check_selected(), // Manual check
                    _ => {}
                }
            }
        }
    }
}

fn ui(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(3),      // Title
                Constraint::Percentage(50), // List
                Constraint::Percentage(50), // Details/Logs
            ]
            .as_ref(),
        )
        .split(f.area());

    // TITLE
    let title = Paragraph::new(Text::from(Line::from(vec![
        Span::styled(
            " 🛡️ SENTINEL CORTEX v8.0 ",
            Style::default()
                .fg(Color::Black)
                .bg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" | MONITOR MODE ", Style::default().fg(Color::Cyan)),
        Span::styled(
            " [R] Refesh  [Enter] Verify Status  [Q] Quit ",
            Style::default().fg(Color::DarkGray),
        ),
    ])))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan)),
    );
    f.render_widget(title, chunks[0]);

    // LIST
    let items: Vec<ListItem> = app
        .operations
        .iter()
        .map(|opt| {
            let s = match opt.status.as_str() {
                "Running" => Span::styled(" ⏳ RUNNING ", Style::default().fg(Color::Yellow)),
                "Completed" => Span::styled(" ✨ DONE    ", Style::default().fg(Color::Green)),
                "Failed" => Span::styled(" ❌ FAILED  ", Style::default().fg(Color::Red)),
                "Lost" => Span::styled(" 🗑️ LOST    ", Style::default().fg(Color::DarkGray)),
                _ => Span::styled(
                    format!(" ?  {} ", opt.status),
                    Style::default().fg(Color::White),
                ),
            };

            ListItem::new(Line::from(vec![
                s,
                Span::raw(" | "),
                Span::styled(
                    opt.target_file.clone(),
                    Style::default().add_modifier(Modifier::BOLD),
                ),
                Span::raw(format!(
                    " ({})",
                    opt.id.chars().rev().take(8).collect::<String>()
                )),
            ]))
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Operations Queue "),
        )
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");
    f.render_stateful_widget(list, chunks[1], &mut app.state);

    // DETAILS / LOG
    let bottom_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(chunks[2]);

    // Left: Selected Details
    let mut details_text = vec![];
    if let Some(i) = app.state.selected() {
        if let Some(op) = app.operations.get(i) {
            if let Some(uri) = &op.gcs_uri {
                details_text.push(Line::from(Span::raw(format!("GCS: {}", uri))));
            }
            if let Some(p) = op.extra.get("prompt") {
                details_text.push(Line::from(Span::raw("--- Prompt ---")));
                details_text.push(Line::from(Span::raw(p.to_string())));
            }
        }
    }
    let details = Paragraph::new(details_text)
        .block(Block::default().borders(Borders::ALL).title(" Metadata "))
        .wrap(Wrap { trim: true });
    f.render_widget(details, bottom_chunks[0]);

    // Right: Logs
    let logs: Vec<ListItem> = app
        .log_buffer
        .iter()
        .map(|l| ListItem::new(Span::raw(l)))
        .collect();
    let log_list = List::new(logs).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" System Logs "),
    );
    f.render_widget(log_list, bottom_chunks[1]);
}
