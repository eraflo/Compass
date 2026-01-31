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
        tx: &Sender<String>,
    ) -> StepStatus {
        // 1. Dependency Validation (Only for shell commands)
        // For other languages (Python, etc.), the content is source code, not a CLI command.
        if language.is_none()
            || matches!(
                language,
                Some("bash" | "sh" | "shell" | "zsh" | "fish" | "cmd" | "powershell")
            )
        {
            if let Err(e) = DependencyValidator::validate(cmd_content) {
                let _ = tx.send(format!("{e}\n"));
                return StepStatus::Failed;
            }
        }

        // 2. Safety Shield (Simple check for now, UI confirmation will be added later)
        if let Some(pattern) = SafetyShield::check(cmd_content) {
            let _ = tx.send(format!(
                "Safety alert: Dangerous pattern detected ('{pattern}').\n"
            ));
            // In the future, this will return a different status to trigger a UI confirmation
            // For now, let's just fail to be safe.
            return StepStatus::Failed;
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
