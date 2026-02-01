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
use crate::core::models::StepStatus;
use crate::ui::app::App;
use crate::ui::state::{ExecutionMessage, Mode};

/// Polls for messages from the execution thread and updates the UI state.
pub fn update(app: &mut App) {
    let messages = app.execution_manager.poll_messages();

    for message in messages {
        match message {
            ExecutionMessage::OutputPartial(i, partial) => {
                if let Some(step) = app.steps.get_mut(i) {
                    crate::ui::utils::append_output(&mut step.output, &partial);

                    if let Some(session) = &app.collab
                        && session.is_host
                        && let Some(tx) = &session.tx
                    {
                        let _ =
                            tx.send(crate::core::collab::events::CompassEvent::OutputReceived {
                                index: i,
                                text: partial.clone(),
                            });
                    }
                }
            }
            ExecutionMessage::Finished(i, status, new_dir, new_env) => {
                let mut recommendation = None;

                let scroll_target = if let Some(step) = app.steps.get_mut(i) {
                    step.status = status;

                    // Broadcast status change if host
                    if let Some(session) = &app.collab
                        && session.is_host
                    {
                        let status_str = match status {
                            StepStatus::Running => "Running",
                            StepStatus::Success => "Success",
                            StepStatus::Failed => "Failed",
                            StepStatus::Skipped => "Skipped",
                            StepStatus::Pending => "Pending",
                        }
                        .to_string();
                        if let Some(tx) = &session.tx {
                            let _ =
                                tx.send(crate::core::collab::events::CompassEvent::StatusChanged {
                                    index: i,
                                    status: status_str,
                                });
                        }
                    }

                    if status == StepStatus::Failed {
                        // Trigger on_failure hook
                        if app.hooks_trusted
                            && let Some(config) = &app.hooks
                        {
                            crate::core::ecosystem::hooks::trigger_hook(
                                &config.on_failure,
                                &new_env,
                            );
                        }
                        recommendation =
                            crate::core::analysis::recovery::analyze_error(&step.output);
                    } else if status == StepStatus::Success {
                        // Trigger on_success hook
                        if app.hooks_trusted
                            && let Some(config) = &app.hooks
                        {
                            crate::core::ecosystem::hooks::trigger_hook(
                                &config.on_success,
                                &new_env,
                            );
                        }
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
pub fn execute_selected(app: &mut App) {
    if let Some(session) = &app.collab
        && !session.is_host
    {
        return; // Guest cannot execute
    }
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
