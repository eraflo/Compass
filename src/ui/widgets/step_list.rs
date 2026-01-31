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

use crate::core::models::{Step, StepStatus};
use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, Borders, List, ListItem, ListState},
};

pub fn render_step_list(frame: &mut Frame, area: Rect, steps: &[Step], list_state: &mut ListState) {
    let items: Vec<ListItem> = steps
        .iter()
        .map(|step| {
            let (symbol, style) = match step.status {
                StepStatus::Running => ("⏳ ", Style::default().fg(Color::Yellow)),
                StepStatus::Success => ("✅ ", Style::default().fg(Color::Green)),
                StepStatus::Failed => ("❌ ", Style::default().fg(Color::Red)),
                StepStatus::Pending => {
                    if step.is_executable() {
                        ("⚡ ", Style::default().fg(Color::Cyan))
                    } else {
                        ("   ", Style::default().fg(Color::Gray))
                    }
                }
            };
            ListItem::new(format!("{symbol}{title}", title = step.title)).style(style)
        })
        .collect();

    let list = List::new(items)
        .block(Block::default().title(" Steps ").borders(Borders::ALL))
        .highlight_style(Style::default().bg(Color::Blue))
        .highlight_symbol(">> ");

    frame.render_stateful_widget(list, area, list_state);
}
