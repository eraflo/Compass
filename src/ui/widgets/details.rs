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
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
};

/// Renders the details panel for the selected step.
///
/// This panel shows:
/// - Step description
/// - Code block(s) with simple syntax highlighting
/// - Execution output with basic ANSI color support
///
/// # Arguments
///
/// * `frame` - The frame to render into.
/// * `area` - The available area for the widget.
/// * `step` - The selected step to display.
/// * `scroll` - The current vertical scroll offset.
///
/// # Returns
///
/// The total height of the content (for scrolling logic).
pub fn render_details(frame: &mut Frame, area: Rect, step: Option<&Step>, scroll: u16) -> u16 {
    let mut text_lines = Vec::new();

    if let Some(step) = step {
        // --- Description ---
        text_lines.push(Line::from(Span::styled(
            &step.description,
            Style::default().fg(Color::White),
        )));
        text_lines.push(Line::from(""));

        // --- Code Blocks ---
        for block in &step.code_blocks {
            let lang = block.language.as_deref().unwrap_or("text");
            // Header
            text_lines.push(Line::from(vec![
                Span::raw("```"),
                Span::styled(
                    lang,
                    Style::default().fg(Color::Yellow).add_modifier(Modifier::ITALIC),
                ),
            ]));

            // Content
            for line in block.content.lines() {
                text_lines.push(Line::from(Span::styled(
                    line,
                    Style::default().fg(Color::Cyan),
                )));
            }

            // Footer
            text_lines.push(Line::from("```"));
            text_lines.push(Line::from(""));
        }

        // --- Output ---
        if !step.output.is_empty() {
            text_lines.push(Line::from(Span::styled(
                "--- Output ---",
                Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD),
            )));
            
            // Simple ANSI simulation
            for line in step.output.lines() {
                // TODO: Implement full ANSI parser here.
                // For now, we do simple heuristic coloring for common log levels
                let style = if line.contains("ERROR") || line.contains("Failed") {
                    Style::default().fg(Color::Red)
                } else if line.contains("WARN") {
                    Style::default().fg(Color::Yellow)
                } else if line.contains("SUCCESS") || line.contains("Done") {
                    Style::default().fg(Color::Green)
                } else {
                    Style::default().fg(Color::Gray)
                };
                
                text_lines.push(Line::from(Span::styled(line, style)));
            }
        }
    } else {
        text_lines.push(Line::from(Span::styled(
            "No step selected.",
            Style::default().fg(Color::DarkGray),
        )));
    }

    // Calculate estimated height (naive wrapping approximation)
    let inner_width = area.width.saturating_sub(2); // borders
    let mut total_lines: u16 = 0;
    if inner_width > 0 {
        for line in &text_lines {
            #[allow(clippy::cast_possible_truncation)]
            let line_len = line.width() as u16; 
            if line_len == 0 {
                total_lines += 1;
            } else {
                total_lines += line_len.div_ceil(inner_width);
            }
        }
    }

    let details = Paragraph::new(text_lines)
        .block(Block::default().title(" Details ").borders(Borders::ALL))
        .wrap(Wrap { trim: true })
        .scroll((scroll, 0));

    frame.render_widget(details, area);

    total_lines
}
