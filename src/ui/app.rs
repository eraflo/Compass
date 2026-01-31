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

use crate::core::executor::{ExecutionContext, Executor};
use crate::core::models::{Step, StepStatus};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
};
use regex::Regex;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;

/// A message sent from the execution thread to the main app.
pub enum ExecutionMessage {
    /// Partial output captured from the PTY.
    OutputPartial(usize, String),
    /// Execution finished with status and final context.
    Finished(usize, StepStatus, PathBuf, HashMap<String, String>),
}

/// The main application state for the TUI.
pub struct App {
    /// The list of steps parsed from the README.
    pub steps: Vec<Step>,
    /// The current state of the step list selection.
    pub list_state: ListState,
    /// Whether the application should exit.
    pub should_quit: bool,
    /// The execution engine.
    pub executor: Executor,
    /// Channel sender for threads.
    pub tx: Sender<ExecutionMessage>,
    /// Channel receiver for the main loop.
    pub rx: Receiver<ExecutionMessage>,
}

impl App {
    pub fn new(steps: Vec<Step>) -> Self {
        let mut list_state = ListState::default();
        if !steps.is_empty() {
            list_state.select(Some(0));
        }
        let (tx, rx) = mpsc::channel();
        Self {
            steps,
            list_state,
            should_quit: false,
            executor: Executor::new(),
            tx,
            rx,
        }
    }

    /// Selects the next step in the list.
    pub fn next(&mut self) {
        let i = match self.list_state.selected() {
            Some(i) => {
                if i >= self.steps.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    /// Selects the previous step in the list.
    pub fn previous(&mut self) {
        let i = match self.list_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.steps.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    /// Renders the TUI.
    pub fn render(&mut self, frame: &mut Frame) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
            .split(frame.size());

        // Sidebar: List of steps
        let items: Vec<ListItem> = self
            .steps
            .iter()
            .map(|step| {
                let symbol = match step.status {
                    StepStatus::Pending => "  ",
                    StepStatus::Running => "⏳ ",
                    StepStatus::Success => "✅ ",
                    StepStatus::Failed => "❌ ",
                };
                let style = match step.status {
                    StepStatus::Running => Style::default().fg(Color::Yellow),
                    StepStatus::Success => Style::default().fg(Color::Green),
                    StepStatus::Failed => Style::default().fg(Color::Red),
                    _ => Style::default().fg(Color::White),
                };
                ListItem::new(format!("{}{}", symbol, step.title)).style(style)
            })
            .collect();

        // Render the list
        let list = List::new(items)
            .block(Block::default().title(" Steps ").borders(Borders::ALL))
            .highlight_style(
                Style::default()
                    .bg(Color::Blue)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol(">> ");

        frame.render_stateful_widget(list, chunks[0], &mut self.list_state);

        // Main panel: Step details
        let selected_index = self.list_state.selected().unwrap_or(0);
        let detail_text = if let Some(step) = self.steps.get(selected_index) {
            let mut content = format!("{}\n\n", step.description);
            for block in &step.code_blocks {
                content.push_str(&format!(
                    "```{}\n{}\n```\n\n",
                    block.language.as_deref().unwrap_or(""),
                    block.content
                ));
            }

            if !step.output.is_empty() {
                content.push_str("--- Output ---\n");
                content.push_str(&step.output);
            }
            content
        } else {
            "No step selected.".to_string()
        };

        // Render the details
        let details = Paragraph::new(detail_text)
            .block(Block::default().title(" Details ").borders(Borders::ALL))
            .wrap(ratatui::widgets::Wrap { trim: true });

        frame.render_widget(details, chunks[1]);
    }

    /// Executes the currently selected step (Non-blocking).
    pub fn execute_selected(&mut self) {
        if let Some(i) = self.list_state.selected() {
            if let Some(step) = self.steps.get(i) {
                if step.status == StepStatus::Running {
                    return; // Already running
                }

                let content = step
                    .code_blocks
                    .iter()
                    .map(|b| b.content.as_str())
                    .collect::<Vec<_>>()
                    .join("\n");

                if content.is_empty() {
                    return;
                }

                let tx = self.tx.clone();
                let current_dir = self.executor.context.current_dir.clone();
                let env_vars = self.executor.context.env_vars.clone();
                let index = i;

                // Mark as running immediately for UI feedback
                self.steps[i].status = StepStatus::Running;
                self.steps[i].output.clear();

                thread::spawn(move || {
                    let mut local_executor = Executor {
                        context: ExecutionContext {
                            current_dir,
                            env_vars,
                        },
                    };
                    let (stream_tx, stream_rx) = mpsc::channel::<String>();

                    let tx_for_streaming = tx.clone();
                    let thread_index = index;

                    // Spawn a sub-thread to forward streaming output to the main app
                    thread::spawn(move || {
                        while let Ok(partial) = stream_rx.recv() {
                            let _ = tx_for_streaming
                                .send(ExecutionMessage::OutputPartial(thread_index, partial));
                        }
                    });

                    let status = local_executor.execute_streamed(&content, stream_tx);

                    let _ = tx.send(ExecutionMessage::Finished(
                        index,
                        status,
                        local_executor.context.current_dir,
                        local_executor.context.env_vars,
                    ));
                });
            }
        }
    }

    /// Polls for messages from the execution thread.
    pub fn update(&mut self) {
        let mut messages = Vec::new();
        while let Ok(message) = self.rx.try_recv() {
            messages.push(message);
        }

        for message in messages {
            match message {
                ExecutionMessage::OutputPartial(i, partial) => {
                    if let Some(step) = self.steps.get_mut(i) {
                        Self::append_output(&mut step.output, &partial);
                    }
                }
                ExecutionMessage::Finished(i, status, new_dir, new_env) => {
                    if let Some(step) = self.steps.get_mut(i) {
                        step.status = status;
                    }
                    // Sync context back to app
                    self.executor.context.current_dir = new_dir;
                    self.executor.context.env_vars = new_env;
                }
            }
        }
    }

    /// Appends output to the buffer, handling carriage returns and ANSI sequences.
    fn append_output(buffer: &mut String, new_data: &str) {
        let cleaned = Self::clean_ansi(new_data);
        for c in cleaned.chars() {
            if c == '\r' {
                // Handle carriage return: truncate the current line
                if let Some(last_newline) = buffer.rfind('\n') {
                    buffer.truncate(last_newline + 1);
                } else {
                    buffer.clear();
                }
            } else {
                buffer.push(c);
            }
        }
    }

    /// Robust ANSI sequence cleaning.
    fn clean_ansi(s: &str) -> String {
        // Corrected regex for ANSI escape sequences:
        // - Escaped [ after the first group: \[
        // - Simplified to be safer and correct
        let re = Regex::new(r"[\x1b\x9b]\[[()#;?]*([0-9A-Za-z;?]*[A-PR-Zcf-ntqry=><~])").unwrap();
        re.replace_all(s, "").to_string()
    }
}
