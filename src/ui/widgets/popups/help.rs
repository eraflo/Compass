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

use crate::ui::utils::centered_rect;
use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
};

pub fn render(frame: &mut Frame, area: Rect, scroll: u16) {
    let area = centered_rect(70, 70, area);
    frame.render_widget(Clear, area);

    let block = Block::default()
        .title(Span::styled(
            " 🧭 Compass - Help ",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let help_items = vec![
        (
            "Navigation",
            vec![
                ("↑ / k", "Move to previous step"),
                ("↓ / j", "Move to next step"),
                ("PgUp / K", "Scroll details up"),
                ("PgDown / J", "Scroll details down"),
            ],
        ),
        (
            "Execution",
            vec![
                ("Enter", "Execute the selected step"),
                ("Esc", "Cancel current modal/action"),
            ],
        ),
        ("Export & Save", vec![("s", "Save/export session report")]),
        (
            "Application",
            vec![("?", "Show this help panel"), ("q", "Quit Compass")],
        ),
    ];

    let mut lines: Vec<Line> = vec![Line::from("")];

    for (section, shortcuts) in help_items {
        // Section header
        lines.push(Line::from(vec![Span::styled(
            format!("  ─── {section} ───"),
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )]));
        lines.push(Line::from(""));

        // Shortcuts
        for (key, description) in shortcuts {
            lines.push(Line::from(vec![
                Span::raw("    "),
                Span::styled(
                    format!("{key:12}"),
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(description, Style::default().fg(Color::White)),
            ]));
        }
        lines.push(Line::from(""));
    }

    // Footer
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "  Press Esc or ? to close this help panel",
        Style::default().fg(Color::DarkGray),
    )));

    let paragraph = Paragraph::new(lines)
        .block(block)
        .wrap(Wrap { trim: false })
        .scroll((scroll, 0));

    frame.render_widget(paragraph, area);
}
