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

use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
};

/// Helper function to create a centered rect of a given size.
pub fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

/// Renders a safety confirmation modal.
pub fn render_safety_alert(frame: &mut Frame, area: Rect, pattern: &str) {
    let area = centered_rect(60, 40, area);
    frame.render_widget(Clear, area); // This clears the area under the popup

    let block = Block::default()
        .title(" ⚠️ SAFETY ALERT ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD));

    let text = format!(
        "\nDangerous pattern detected:\n\n  '{pattern}'\n\nThis command could damage your system.\n\nPress [Enter] to execute anyway, or [Esc] to cancel."
    );

    let paragraph = Paragraph::new(text)
        .block(block)
        .wrap(Wrap { trim: true })
        .alignment(ratatui::layout::Alignment::Center);

    frame.render_widget(paragraph, area);
}

/// Renders an input modal for placeholders.
pub fn render_input_modal(frame: &mut Frame, area: Rect, var_name: &str, current_input: &str) {
    let area = centered_rect(60, 30, area);
    frame.render_widget(Clear, area);

    // Dynamic title to ensure visibility of input
    let title = format!(" [ Input: {var_name} ] (Typing: \"{current_input}\") ");

    let block = Block::default()
        .title(Span::styled(
            title,
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
        .border_style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        );

    let text = vec![
        Line::from(""),
        Line::from(vec![
            Span::raw("  Please provide a value for: "),
            Span::styled(
                var_name,
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::raw("  > "),
            Span::styled(
                current_input,
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("█", Style::default().fg(Color::White)),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "  (Enter: Confirm | Esc: Cancel)",
            Style::default().fg(Color::DarkGray),
        )),
    ];

    let paragraph = Paragraph::new(text).block(block).wrap(Wrap { trim: false });

    frame.render_widget(paragraph, area);
}

/// Renders the help modal with all keyboard shortcuts.
///
/// This modal displays a comprehensive list of all available commands
/// and their corresponding keyboard shortcuts.
pub fn render_help_modal(frame: &mut Frame, area: Rect, scroll: u16) {
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
        ("Navigation", vec![
            ("↑ / k", "Move to previous step"),
            ("↓ / j", "Move to next step"),
            ("PgUp / K", "Scroll details up"),
            ("PgDown / J", "Scroll details down"),
        ]),
        ("Execution", vec![
            ("Enter", "Execute the selected step"),
            ("Esc", "Cancel current modal/action"),
        ]),
        ("Export & Save", vec![
            ("s", "Save/export session report"),
        ]),
        ("Application", vec![
            ("?", "Show this help panel"),
            ("q", "Quit Compass"),
        ]),
    ];

    let mut lines: Vec<Line> = vec![Line::from("")];

    for (section, shortcuts) in help_items {
        // Section header
        lines.push(Line::from(vec![
            Span::styled(
                format!("  ─── {section} ───"),
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
        ]));
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

/// Renders an export notification modal.
///
/// Displays a success or error message after an export operation.
///
/// # Arguments
///
/// * `frame` - The frame to render into.
/// * `area` - The area available for rendering.
/// * `success` - Whether the export was successful.
/// * `message` - The message to display (file path or error).
pub fn render_export_notification(frame: &mut Frame, area: Rect, success: bool, message: &str) {
    let area = centered_rect(60, 30, area);
    frame.render_widget(Clear, area);

    let (title, title_color, border_color) = if success {
        (" ✅ Export Successful ", Color::Green, Color::Green)
    } else {
        (" ❌ Export Failed ", Color::Red, Color::Red)
    };

    let block = Block::default()
        .title(Span::styled(
            title,
            Style::default()
                .fg(title_color)
                .add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color));

    let text = vec![
        Line::from(""),
        Line::from(vec![
            Span::raw("  "),
            Span::styled(
                if success { "Report saved to:" } else { "Error:" },
                Style::default().fg(Color::White),
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::raw("  "),
            Span::styled(
                message,
                Style::default()
                    .fg(if success { Color::Cyan } else { Color::Red })
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "  Press any key to continue...",
            Style::default().fg(Color::DarkGray),
        )),
    ];

    let paragraph = Paragraph::new(text).block(block).wrap(Wrap { trim: false });

    frame.render_widget(paragraph, area);
}
