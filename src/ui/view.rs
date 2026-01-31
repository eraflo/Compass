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

use crate::ui::app::App;
use crate::ui::state::Mode;
use crate::ui::widgets::{details, popups, step_list};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
};

/// Renders the UI.
pub fn draw(frame: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(33), Constraint::Percentage(67)])
        .split(frame.size());

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

    // Render modals if active
    match app.mode {
        Mode::InputModal => {
            // Use modal state
            if let Some(var_name) = app
                .modal
                .required_placeholders
                .get(app.modal.current_placeholder_idx)
            {
                popups::render_input_modal(frame, frame.size(), var_name, &app.modal.input_buffer);
            }
        }
        Mode::SafetyAlert => {
            if let Some(ref pattern) = app.safety_pattern {
                popups::render_safety_alert(frame, frame.size(), pattern);
            }
        }
        Mode::Normal => {}
    }
}
