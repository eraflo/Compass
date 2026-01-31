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

use crate::ui::app::{App, VERSION};
use crate::ui::state::Mode;
use crate::ui::widgets::{details, popups, step_list};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
};

/// Renders the status bar at the bottom of the screen.
///
/// The status bar displays:
/// - Compass version
/// - Progress summary (completed/total steps)
/// - Quick help hint
fn render_status_bar(frame: &mut Frame, area: Rect, app: &App) {
    let completed = app.completed_count();
    let failed = app.failed_count();
    let total = app.total_executable_steps();

    // Build status line with multiple spans
    let mut spans = vec![
        Span::styled(
            format!(" 🧭 Compass v{VERSION} "),
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("│", Style::default().fg(Color::DarkGray)),
    ];

    if app.is_remote {
        spans.push(Span::styled(
            " 🌐 Remote ",
            Style::default()
                .fg(Color::Magenta)
                .add_modifier(Modifier::BOLD),
        ));
        spans.push(Span::styled("│", Style::default().fg(Color::DarkGray)));
    }

    if app.is_sandbox() {
        spans.push(Span::styled(
            " 📦 SANDBOXED ",
            Style::default()
                .fg(Color::LightCyan)
                .add_modifier(Modifier::BOLD),
        ));
        spans.push(Span::styled("│", Style::default().fg(Color::DarkGray)));
    }

    spans.extend(vec![
        Span::styled(
            format!(" ✅ {completed}/{total} "),
            Style::default().fg(Color::Green),
        ),
        if failed > 0 {
            Span::styled(format!("❌ {failed} "), Style::default().fg(Color::Red))
        } else {
            Span::raw("")
        },
        Span::styled("│", Style::default().fg(Color::DarkGray)),
        Span::styled(" ? Help ", Style::default().fg(Color::Yellow)),
        Span::styled("│ s Save │ q Quit ", Style::default().fg(Color::DarkGray)),
    ]);

    let status_line = Line::from(spans);

    let status_bar = Paragraph::new(status_line).style(Style::default().bg(Color::Rgb(30, 30, 40)));

    frame.render_widget(status_bar, area);
}

/// Renders the UI.
///
/// This function is responsible for drawing all UI components:
/// - Step list (left panel)
/// - Details panel (right panel)
/// - Status bar (bottom)
/// - Modal popups (overlays)
pub fn draw(frame: &mut Frame, app: &mut App) {
    // Main layout: content area + status bar
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(1)])
        .split(frame.area());

    let content_area = main_chunks[0];
    let status_area = main_chunks[1];

    // Content layout: step list + details
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(33), Constraint::Percentage(67)])
        .split(content_area);

    // Render the step list
    step_list::render_step_list(frame, chunks[0], &app.steps, &mut app.list_state);

    // Render the details
    let selected_index = app.list_state.selected().unwrap_or(0);
    app.content_height = details::render_details(
        frame,
        chunks[1],
        app.steps.get(selected_index),
        app.details_scroll,
    );
    app.viewport_height = chunks[1].height.saturating_sub(2);

    // Render the status bar
    render_status_bar(frame, status_area, app);

    // Render modals if active
    match app.mode {
        Mode::InputModal => {
            // Use modal state
            if let Some(var_name) = app
                .modal
                .required_placeholders
                .get(app.modal.current_placeholder_idx)
            {
                popups::input::render(frame, frame.area(), var_name, &app.modal.input_buffer);
            }
        }
        Mode::SafetyAlert => {
            if let Some(ref pattern) = app.safety_pattern {
                popups::safety::render(frame, frame.area(), pattern);
            }
        }
        Mode::DependencyAlert => {
            if let Some(ref message) = app.safety_pattern {
                popups::dependency::render(frame, frame.area(), message);
            }
        }
        Mode::HelpModal => {
            popups::help::render(frame, frame.area(), app.help_scroll);
        }
        Mode::ExportNotification => {
            if let Some((success, ref message)) = app.export_message {
                popups::notification::render(frame, frame.area(), success, message);
            }
        }
        Mode::RecoveryAlert => {
            if let Some(ref rec) = app.recovery_suggestion {
                popups::recovery::render(frame, frame.area(), rec);
            }
        }
        Mode::Normal => {}
    }
}
