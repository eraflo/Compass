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
use ansi_to_tui::IntoText;
use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
};
use std::sync::OnceLock;
use syntect::easy::HighlightLines;
use syntect::highlighting::ThemeSet;
use syntect::parsing::SyntaxSet;

static SYNTAX_SET: OnceLock<SyntaxSet> = OnceLock::new();
static THEME_SET: OnceLock<ThemeSet> = OnceLock::new();

fn get_syntax_set() -> &'static SyntaxSet {
    SYNTAX_SET.get_or_init(SyntaxSet::load_defaults_newlines)
}

fn get_theme_set() -> &'static ThemeSet {
    THEME_SET.get_or_init(ThemeSet::load_defaults)
}

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
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::ITALIC),
                ),
            ]));

            // Prepare highlighter
            let ps = get_syntax_set();
            let ts = get_theme_set();
            let syntax = ps
                .find_syntax_by_token(lang)
                .unwrap_or_else(|| ps.find_syntax_plain_text());

            // Use a dark theme that contrasts well with standard terminal backgrounds
            let theme = &ts
                .themes
                .get("base16-ocean.dark")
                .or_else(|| ts.themes.get("base16-mocha.dark"))
                .unwrap_or_else(|| ts.themes.values().next().unwrap());
            let mut h = HighlightLines::new(syntax, theme);

            // Content
            for line in block.content.lines() {
                // Syntect expects standard Rust strings, but technically prefers newlines for context.
                // However, for single-pass highlighting of lines, this works well enough for display.
                let ranges = h.highlight_line(line, ps).unwrap_or_default();

                let spans: Vec<Span> = ranges
                    .into_iter()
                    .map(|(style, text)| {
                        let fg =
                            Color::Rgb(style.foreground.r, style.foreground.g, style.foreground.b);
                        Span::styled(text, Style::default().fg(fg))
                    })
                    .collect();

                text_lines.push(Line::from(spans));
            }

            // Footer
            text_lines.push(Line::from("```"));
            text_lines.push(Line::from(""));
        }

        // --- Output ---
        let raw_output = &step.output;
        let trimmed_output = raw_output.trim();

        if !trimmed_output.is_empty() {
            text_lines.push(Line::from(Span::styled(
                "--- Output ---",
                Style::default()
                    .fg(Color::Magenta)
                    .add_modifier(Modifier::BOLD),
            )));

            // Render ANSI output using ansi-to-tui
            match trimmed_output.as_bytes().into_text() {
                Ok(output_text) => {
                    text_lines.extend(output_text.lines);
                }
                Err(_) => {
                    // Fallback to plain text if parsing fails
                    for line in trimmed_output.lines() {
                        text_lines.push(Line::from(Span::styled(
                            line,
                            Style::default().fg(Color::Gray),
                        )));
                    }
                }
            }
        }
    } else {
        text_lines.push(Line::from(Span::styled(
            "No step selected.",
            Style::default().fg(Color::DarkGray),
        )));
    }

    // Calculate estimated height (naive wrapping approximation)
    // We add a safety margin because simple char-counting underestimates
    // height when word-wrapping occurs (ratatui wraps at spaces).
    let inner_width = area.width.saturating_sub(2); // borders
    let mut total_lines: u16 = 0;
    if inner_width > 0 {
        for line in &text_lines {
            #[allow(clippy::cast_possible_truncation)]
            let line_len = line.width() as u16;
            if line_len == 0 {
                total_lines += 1;
            } else {
                // Heuristic: Add 10% extra for word-wrapping inefficiencies
                // plus strict calculation
                total_lines += line_len.div_ceil(inner_width);
            }
        }
        // Add a small constant buffer at the end to ensure last lines are visible
        total_lines += 2;
    }

    let details = Paragraph::new(text_lines)
        .block(Block::default().title(" Details ").borders(Borders::ALL))
        .wrap(Wrap { trim: true })
        .scroll((scroll, 0));

    frame.render_widget(details, area);

    total_lines
}
