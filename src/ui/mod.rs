// Copyright 2026 eraflo
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

pub mod app;
pub mod events;
pub mod state;
pub mod utils;
pub mod view;
pub mod widgets;

use crate::core::models::Step;
use crate::ui::app::App;
use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyEventKind},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use std::io;
use std::path::PathBuf;

/// Starts the TUI application.
pub fn run_tui(
    steps: Vec<Step>,
    readme_path: PathBuf,
    is_remote: bool,
    sandbox: bool,
    image: String,
    collab_session: Option<crate::core::collab::session::CollabSession>,
) -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?; // Enter alternate screen
    let backend = CrosstermBackend::new(stdout); // Create backend
    let mut terminal = Terminal::new(backend)?; // Create terminal

    // Create app and run main loop
    let mut app = App::new(steps, readme_path, is_remote).with_sandbox(sandbox, image);

    if let Some(session) = collab_session {
        app.collab = Some(session);
    }

    // Load persisted configuration (placeholders)
    app.load_config();

    let res = run_loop(&mut terminal, app);

    // Restore terminal
    disable_raw_mode()?; // Disable raw mode
    execute!(terminal.backend_mut(), LeaveAlternateScreen,)?; // Leave alternate screen
    terminal.show_cursor()?; // Show cursor

    res
}

/// Runs the main loop of the TUI application.
fn run_loop(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, mut app: App) -> Result<()> {
    // Send initial snapshot if host
    if let Some(session) = &app.collab
        && session.is_host
        && let Some(tx) = &session.tx
    {
        let event = crate::core::collab::events::CompassEvent::Snapshot {
            steps: app.steps.clone(),
            current_step: app.list_state.selected().unwrap_or(0),
        };
        let _ = tx.send(event);
    }

    loop {
        terminal.draw(|f| view::draw(f, &mut app))?;

        // Handle incoming collab events
        let mut events_to_process = Vec::new();
        if let Some(session) = &app.collab
            && let Some(rx) = &session.rx
        {
            while let Ok(event) = rx.try_recv() {
                events_to_process.push(event);
            }
        }

        for event in events_to_process {
            match event {
                crate::core::collab::events::CompassEvent::StepChanged(idx) => {
                    app.list_state.select(Some(idx));
                }
                crate::core::collab::events::CompassEvent::StatusChanged { index, status } => {
                    if let Some(step) = app.steps.get_mut(index) {
                        step.status = match status.as_str() {
                            "Running" => crate::core::models::StepStatus::Running,
                            "Success" => crate::core::models::StepStatus::Success,
                            "Failed" => crate::core::models::StepStatus::Failed,
                            "Skipped" => crate::core::models::StepStatus::Skipped,
                            _ => crate::core::models::StepStatus::Pending,
                        };
                    }
                }
                crate::core::collab::events::CompassEvent::OutputReceived { index, text } => {
                    if let Some(step) = app.steps.get_mut(index) {
                        crate::ui::utils::append_output(&mut step.output, &text);
                    }
                }
                crate::core::collab::events::CompassEvent::Snapshot {
                    steps,
                    current_step,
                } => {
                    // Full sync if re-connecting or initial
                    app.steps = steps;
                    app.list_state.select(Some(current_step));
                }
                crate::core::collab::events::CompassEvent::ConnectionLost(msg) => {
                    return Err(anyhow::anyhow!("Session disconnected: {}", msg));
                }
            }
        }

        if matches!(event::poll(std::time::Duration::from_millis(100)), Ok(true)) {
            #[allow(clippy::collapsible_if)]
            if let Ok(Event::Key(key)) = event::read() {
                if key.kind == KeyEventKind::Press {
                    events::input::handle_input(&mut app, key);
                }
            }
        }

        events::handlers::update(&mut app);

        if app.should_quit {
            return Ok(());
        }
    }
}
