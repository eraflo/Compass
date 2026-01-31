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
    layout::Rect,
    widgets::{Block, Borders, Paragraph, Wrap},
};
use std::fmt::Write;

pub fn render_details(frame: &mut Frame, area: Rect, step: Option<&Step>, scroll: u16) -> u16 {
    let detail_text = step.map_or_else(
        || "No step selected.".to_string(),
        |step| {
            let mut content = format!("{}\n\n", step.description);
            for block in &step.code_blocks {
                let lang = block.language.as_deref().unwrap_or("");
                let _ = write!(content, "```{}\n{}\n```\n\n", lang, block.content);
            }

            if !step.output.is_empty() {
                content.push_str("--- Output ---\n");
                content.push_str(&step.output);
            }
            content
        },
    );

    // Calculate estimated height
    let inner_width = area.width.saturating_sub(2); // borders
    let mut total_lines = 0;
    if inner_width > 0 {
        for line in detail_text.lines() {
            let line_len = line.chars().count() as u16;
            if line_len == 0 {
                total_lines += 1;
            } else {
                total_lines += (line_len + inner_width - 1) / inner_width;
            }
        }
    }

    let details = Paragraph::new(detail_text)
        .block(Block::default().title(" Details ").borders(Borders::ALL))
        .wrap(Wrap { trim: true })
        .scroll((scroll, 0));

    frame.render_widget(details, area);

    total_lines as u16
}
