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

use crate::core::executor::{CommandBuilder, SafetyShield};
use crate::core::models::StepStatus;
use crate::ui::app::App;
use crate::ui::state::{ExecutionMessage, Mode};

/// Handles submission of a placeholder value from the input modal.
pub fn submit_input(app: &mut App) {
    if app.mode != Mode::InputModal {
        return;
    }

    let var_name = app.modal.required_placeholders[app.modal.current_placeholder_idx].clone();
    let value = app.modal.input_buffer.clone();

    app.modal.variable_store.insert(var_name, value);

    app.modal.current_placeholder_idx += 1;
    app.modal.input_buffer.clear();

    if app.modal.current_placeholder_idx < app.modal.required_placeholders.len() {
        // Pre-fill next variable
        let next_var = &app.modal.required_placeholders[app.modal.current_placeholder_idx];
        app.modal.input_buffer = app
            .modal
            .variable_store
            .get(next_var)
            .cloned()
            .unwrap_or_default();
    } else {
        // All filled, now try to execute
        app.mode = Mode::Normal;
        perform_execution(app, false);
    }
}

/// Polls for messages from the execution thread and updates the UI state.
pub fn update(app: &mut App) {
    let messages = app.execution_manager.poll_messages();

    for message in messages {
        match message {
            ExecutionMessage::OutputPartial(i, partial) => {
                if let Some(step) = app.steps.get_mut(i) {
                    crate::ui::utils::append_output(&mut step.output, &partial);
                }
            }
            ExecutionMessage::Finished(i, status, new_dir, new_env) => {
                let scroll_target = if let Some(step) = app.steps.get_mut(i) {
                    step.status = status;
                    let finish_status = match status {
                        StepStatus::Success => "✅ Execution finished successfully.",
                        StepStatus::Failed => "❌ Execution failed.",
                        _ => "",
                    };
                    step.output.push_str("\n\n---\n");
                    step.output.push_str(finish_status);

                    let description_height = step.description.lines().count() + 2;
                    let code_blocks_height: usize = step
                        .code_blocks
                        .iter()
                        .map(|b| b.content.lines().count() + 2)
                        .sum();

                    #[allow(clippy::cast_possible_truncation)]
                    let target = (description_height + code_blocks_height + 2) as u16;
                    target
                } else {
                    0
                };

                app.details_scroll = scroll_target;
                app.execution_manager.executor.context.current_dir = new_dir;
                app.execution_manager.executor.context.env_vars = new_env;
            }
        }
    }
}

/// Executes the currently selected step (Non-blocking).
pub fn execute_selected(app: &mut App) {
    perform_execution(app, false);
}

/// Internal helper to handle execution flow with safety checks.
pub fn perform_execution(app: &mut App, bypass_safety: bool) {
    if app.mode != Mode::Normal {
        return;
    }

    if let Some(i) = app.list_state.selected() {
        // Check if already running
        if let Some(step) = app.steps.get(i) {
            #[allow(clippy::collapsible_if)]
            if step.status == StepStatus::Running {
                return;
            }
        }

        // Check if we need to prompt for placeholders.
        let step_placeholders = CommandBuilder::get_required_placeholders(&app.steps[i]);

        if !step_placeholders.is_empty() && app.modal.required_placeholders.is_empty() {
            app.modal.reset(step_placeholders);

            // Pre-fill with previous value if exists
            if !app.modal.required_placeholders.is_empty() {
                let first_var = &app.modal.required_placeholders[0];
                app.modal.input_buffer = app
                    .modal
                    .variable_store
                    .get(first_var)
                    .cloned()
                    .unwrap_or_default();
            }

            app.mode = Mode::InputModal;
            return;
        }

        let content = CommandBuilder::build_command(&app.steps[i], &app.modal.variable_store);
        app.modal.required_placeholders.clear();

        if content.trim().is_empty() {
            return;
        }

        // Safety Shield
        if !bypass_safety {
            #[allow(clippy::collapsible_if)]
            if let Some(pattern) = SafetyShield::check(&content) {
                app.safety_pattern = Some(pattern.to_string());
                app.mode = Mode::SafetyAlert;
                return;
            }
        }

        // Execute background
        app.steps[i].status = StepStatus::Running;
        app.steps[i].output = String::new();
        app.execution_manager.execute_background(i, content);
    }
}

/// Confirms execution of a dangerous command.
pub fn confirm_safety(app: &mut App) {
    if app.mode != Mode::SafetyAlert {
        return;
    }
    app.mode = Mode::Normal;
    app.safety_pattern = None;
    perform_execution(app, true);
}
