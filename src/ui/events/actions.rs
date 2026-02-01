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

use super::execution::perform_execution;
use crate::core::export::Exporter;
use crate::ui::app::{App, VERSION};
use crate::ui::state::Mode;

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
        // All filled, save config and execute
        app.save_config();
        app.mode = Mode::Normal;
        perform_execution(app, false);
    }
}

/// Confirms execution of a dangerous command.
pub fn confirm_safety(app: &mut App) {
    if app.mode != Mode::SafetyAlert && app.mode != Mode::DependencyAlert {
        return;
    }
    app.mode = Mode::Normal;
    app.safety_pattern = None;
    perform_execution(app, true);
}

/// Handles interaction with the recovery alert modal.
#[allow(clippy::collapsible_if)]
pub fn confirm_recovery(app: &mut App) {
    if app.mode != Mode::RecoveryAlert {
        return;
    }

    // Try to run the fix if available
    if let Some(rec) = &app.recovery_suggestion {
        if let Some(cmd) = &rec.fix_command {
            // Find current step index
            if let Some(i) = app.list_state.selected() {
                app.execution_manager.execute_background(
                    i,
                    cmd.clone(),
                    Some("bash".to_string()),
                    true,
                );
                // We don't perform full execution, just run the fix
            }
        }
    }

    // If no fix or just info, just close
    app.mode = Mode::Normal;
    app.recovery_suggestion = None;
}

/// Exports the current session to JSON and Markdown files.
pub fn export_report(app: &mut App) {
    if app.mode != Mode::Normal {
        return;
    }

    // Generate the report
    let report = Exporter::generate_report(
        &app.steps,
        &app.readme_path,
        &app.execution_manager.executor.context.current_dir,
        &app.execution_manager.executor.context.env_vars,
        &app.modal.variable_store,
        VERSION,
    );

    // Get the base directory (current working directory)
    let base_dir = &app.execution_manager.executor.context.current_dir;

    // Export to both formats
    match Exporter::export_both(&report, base_dir) {
        Ok((json_path, md_path)) => {
            let message = format!("{}\n{}", json_path.display(), md_path.display());
            app.export_message = Some((true, message));
            app.mode = Mode::ExportNotification;
        }
        Err(e) => {
            app.export_message = Some((false, e.to_string()));
            app.mode = Mode::ExportNotification;
        }
    }
}
