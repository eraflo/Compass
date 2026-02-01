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

use crate::core::executor::engine::builtin::BuiltinHandler;
use crate::core::executor::engine::context::ExecutionContext;
use crate::core::executor::engine::session::ShellSession;
use crate::core::executor::languages::get_language_handler;
use crate::core::executor::security::safety::SafetyShield;
use crate::core::executor::security::validator::DependencyValidator;
use crate::core::models::StepStatus;
use std::sync::mpsc::Sender;

/// The main entry point for the execution engine.
pub struct Executor {
    pub context: ExecutionContext,
}

impl Executor {
    /// Creates a new `Executor` with a default `ExecutionContext`.
    #[must_use]
    pub fn new() -> Self {
        Self {
            context: ExecutionContext::new(),
        }
    }

    /// Orchestrates the execution of a code block.
    pub fn execute_streamed(
        &mut self,
        cmd_content: &str,
        language: Option<&str>,
        bypass_safety: bool,
        tx: &Sender<String>,
    ) -> StepStatus {
        // 1. Dependency Validation
        // This acts as a final enforcement. The UI should have already prompted the user,
        // so if we are here with bypass_safety=false and it fails, it means we are in headless mode
        // or something went wrong. We return Failed.

        if !bypass_safety {
            let is_shell = language.is_none()
                || matches!(
                    language,
                    Some("bash" | "sh" | "shell" | "zsh" | "fish" | "cmd" | "powershell")
                );

            if is_shell {
                if let Err(e) = DependencyValidator::validate(cmd_content) {
                    let _ = tx.send(format!("{e}\n"));
                    return StepStatus::Failed;
                }
            } else {
                let handler = get_language_handler(language);
                let required_cmd = handler.get_required_command();
                if let Err(e) = DependencyValidator::validate_binary(required_cmd) {
                    let _ = tx.send(format!("{e}\n"));
                    return StepStatus::Failed;
                }
            }
        }

        // 2. Safety Shield
        if !bypass_safety {
            let handler = get_language_handler(language);
            let patterns = handler.get_dangerous_patterns();

            if let Some(pattern) = SafetyShield::check(cmd_content, patterns) {
                let _ = tx.send(format!(
                    "Safety alert: Dangerous pattern detected ('{pattern}'). Execution blocked.\n"
                ));
                // The UI handles the confirmation dialog before calling this with bypass_safety=true.
                // If we reach here, it means the check failed and was not bypassed (e.g. headless run).
                return StepStatus::Failed;
            }
        }

        // 3. Handle side-effects (builtins)
        let (cleaned_content, simulated_output) =
            BuiltinHandler::process(cmd_content, &mut self.context);

        if !simulated_output.is_empty() {
            let _ = tx.send(simulated_output);
        }

        if cleaned_content.trim().is_empty() {
            return StepStatus::Success;
        }

        // 4. Run via ShellSession
        let session = ShellSession::new(self.context.clone());
        session.run(&cleaned_content, language, tx)
    }
}
