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

use crate::core::models::StepStatus;
use std::path::PathBuf;
use std::process::Command;

/// Handles the execution of code blocks and maintains the working directory.
pub struct ExecutionContext {
    pub current_dir: PathBuf,
}

impl ExecutionContext {
    pub fn new() -> Self {
        Self {
            current_dir: std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
        }
    }

    /// Executes a code block and returns the status and captured output.
    pub fn execute(&mut self, content: &str) -> (StepStatus, String) {
        let mut output_buf = String::new();
        let mut final_status = StepStatus::Success;

        // Split content into lines and process them
        // For now, we execute the whole block as a single shell command
        // This handles multiple commands if they are valid shell (e.g., cmd1 && cmd2)

        // Check if there are 'cd' commands to update our state
        // A more robust way would be to wrap the command and export the PWD,
        // but for a simple version we can try to detect 'cd' at the start of lines.
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("cd ") {
                let path_str = &trimmed[3..].trim().replace("\"", "").replace("'", "");
                let new_path = self.current_dir.join(path_str);
                if new_path.exists() && new_path.is_dir() {
                    self.current_dir = new_path.canonicalize().unwrap_or(self.current_dir.clone());
                }
            }
        }

        let cmd_result = if cfg!(target_os = "windows") {
            Command::new("powershell")
                .arg("-Command")
                .arg(content)
                .current_dir(&self.current_dir)
                .output()
        } else {
            Command::new("sh")
                .arg("-c")
                .arg(content)
                .current_dir(&self.current_dir)
                .output()
        };

        match cmd_result {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let stderr = String::from_utf8_lossy(&output.stderr);

                output_buf.push_str(&stdout);
                output_buf.push_str(&stderr);

                if !output.status.success() {
                    final_status = StepStatus::Failed;
                }
            }
            Err(e) => {
                output_buf.push_str(&format!("Error executing command: {}", e));
                final_status = StepStatus::Failed;
            }
        }

        (final_status, output_buf)
    }
}
