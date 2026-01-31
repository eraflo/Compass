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

use crate::core::executor::engine::CommandBuilder;
use crate::core::executor::languages::get_language_handler;
use crate::core::executor::security::safety::SafetyShield;
use crate::core::executor::security::validator::DependencyValidator;
use crate::core::export::Exporter;
use crate::core::models::StepStatus;
use crate::ui::app::{App, VERSION};
use crate::ui::state::{ExecutionMessage, Mode};

/// Handles submission of a placeholder value from the input modal.
///
/// When the user presses Enter in the input modal, this function
/// stores the value and either moves to the next placeholder or
/// triggers execution if all placeholders have been filled.
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

/// Polls for messages from the execution thread and updates the UI state.
///
/// This function should be called regularly in the main loop to process
/// execution updates (output and completion status).
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
                let mut recommendation = None;

                let scroll_target = if let Some(step) = app.steps.get_mut(i) {
                    step.status = status;

                    if status == StepStatus::Failed {
                        recommendation =
                            crate::core::analysis::recovery::analyze_error(&step.output);
                    }

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

                if let Some(rec) = recommendation {
                    app.recovery_suggestion = Some(rec);
                    app.mode = crate::ui::state::Mode::RecoveryAlert;
                }

                app.details_scroll = scroll_target;
                app.execution_manager.executor.context.current_dir = new_dir;
                app.execution_manager.executor.context.env_vars = new_env;
            }
        }
    }
}

/// Executes the currently selected step (Non-blocking).
///
/// This is the main entry point for step execution. It handles placeholder
/// prompts, safety checks, and spawns the background execution.
pub fn execute_selected(app: &mut App) {
    perform_execution(app, false);
}

/// Internal helper to handle execution flow with safety checks.
///
/// # Arguments
///
/// * `app` - The application state.
/// * `bypass_safety` - If true, skip safety checks (used after user confirmation).
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

        // Check conditions
        let should_skip = if let Some(step) = app.steps.get(i) {
            if let Some(condition) = &step.condition {
                use crate::core::executor::conditions::evaluator::{
                    ConditionEvaluator, StandardEvaluator,
                };
                let evaluator = StandardEvaluator::new();
                !evaluator.evaluate(condition)
            } else {
                false
            }
        } else {
            false
        };

        if should_skip {
            if let Some(step) = app.steps.get_mut(i) {
                step.status = StepStatus::Skipped;
                step.output.push_str("\n> ⏭️ Skipped: Condition not met.\n");
            }
            return;
        }

        // Check if we need to prompt for placeholders.
        let step_placeholders = CommandBuilder::get_required_placeholders(&app.steps[i]);

        if !step_placeholders.is_empty() && app.modal.required_placeholders.is_empty() {
            app.modal.reset(step_placeholders);

            // Pre-fill with previous value if exists (from config or previous input)
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

        let language = app.steps[i]
            .code_blocks
            .first()
            .and_then(|cb| cb.language.as_deref())
            .map(ToString::to_string);

        // Safety Checks
        if !bypass_safety {
            // 1. Dependency Check
            let is_shell = language.is_none()
                || matches!(
                    language.as_deref(),
                    Some("bash" | "sh" | "shell" | "zsh" | "fish" | "cmd" | "powershell")
                );

            if is_shell {
                if let Err(e) = DependencyValidator::validate(&content) {
                    app.safety_pattern = Some(e);
                    app.mode = Mode::DependencyAlert;
                    return;
                }
            } else {
                // For other languages, check if the interpreter is installed
                let handler = get_language_handler(language.as_deref());
                let required_cmd = handler.get_required_command();
                if let Err(e) = DependencyValidator::validate_binary(required_cmd) {
                    app.safety_pattern = Some(e);
                    app.mode = Mode::DependencyAlert;
                    return;
                }
            }

            // 2. Dangerous Patterns
            let handler = get_language_handler(language.as_deref());
            let patterns = handler.get_dangerous_patterns();
            let check_result = SafetyShield::check(&content, patterns);

            if app.is_remote {
                app.safety_pattern = Some(
                    check_result
                        .map(ToString::to_string)
                        .unwrap_or_else(|| "Remote Source (Strict Mode)".to_string()),
                );
                app.mode = Mode::SafetyAlert;
                return;
            }

            #[allow(clippy::collapsible_if)]
            if let Some(pattern) = check_result {
                app.safety_pattern = Some(pattern.to_string());
                app.mode = Mode::SafetyAlert;
                return;
            }
        }

        // Execute background
        app.steps[i].status = StepStatus::Running;
        app.steps[i].output = String::new();
        app.execution_manager
            .execute_background(i, content, language, bypass_safety);
    }
}

/// Confirms execution of a dangerous command.
///
/// Called when the user presses Enter on the safety alert modal.
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
            if let Some(i) = app.list_state.selected() {
                // Close modal and run fix
                app.mode = Mode::Normal;
                let cmd_clone: String = cmd.clone(); // Clone before mutation

                // Clear suggestion
                app.recovery_suggestion = None;

                // Update step status
                if let Some(step) = app.steps.get_mut(i) {
                    step.status = StepStatus::Running;
                    step.output
                        .push_str(&format!("\n\n> 💡 Auto-Fix: {}\n", cmd_clone));
                }

                // Execute the fix
                app.execution_manager
                    .execute_background(i, cmd_clone, None, false);
                return;
            }
        }
    }

    // If no fix or just info, just close
    app.mode = Mode::Normal;
    app.recovery_suggestion = None;
}

/// Exports the current session to JSON and Markdown files.
///
/// The files are saved to the current working directory with timestamped names.
/// The result (success or failure) is displayed in a notification modal.
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
