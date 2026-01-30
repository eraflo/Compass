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

use crate::core::models::Step;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
};

/// The main application state for the TUI.
pub struct App {
    /// The list of steps parsed from the README.
    pub steps: Vec<Step>,
    /// The current state of the step list selection.
    pub list_state: ListState,
    /// Whether the application should exit.
    pub should_quit: bool,
}

impl App {
    pub fn new(steps: Vec<Step>) -> Self {
        let mut list_state = ListState::default();
        if !steps.is_empty() {
            list_state.select(Some(0));
        }
        Self {
            steps,
            list_state,
            should_quit: false,
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
            .map(|step| ListItem::new(step.title.clone()).style(Style::default().fg(Color::White)))
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
}
