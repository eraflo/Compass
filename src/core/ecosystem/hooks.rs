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

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::process::Command;
use std::thread;

/// Configuration for event hooks extracted from frontmatter.
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct HookConfig {
    pub pre_run: Option<String>,
    pub post_run: Option<String>,
    pub on_failure: Option<String>,
    pub on_success: Option<String>,
}

impl HookConfig {
    pub fn has_any(&self) -> bool {
        self.pre_run.is_some()
            || self.post_run.is_some()
            || self.on_failure.is_some()
            || self.on_success.is_some()
    }
}

/// Triggers a hook command in a background thread.
///
/// # Arguments
/// * `hook_cmd` - The shell command to execute.
/// * `context_env` - Environment variables to inject into the command.
pub fn trigger_hook(hook_cmd: &Option<String>, context_env: &HashMap<String, String>) {
    if let Some(cmd) = hook_cmd {
        let cmd_string = cmd.clone();
        let envs: Vec<(String, String)> = context_env
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();

        // Spawn a thread to avoid blocking the main UI loop
        thread::spawn(move || {
            #[cfg(target_os = "windows")]
            let mut command = Command::new("powershell");
            #[cfg(target_os = "windows")]
            command.args(["-Command", &cmd_string]);

            #[cfg(not(target_os = "windows"))]
            let mut command = Command::new("sh");
            #[cfg(not(target_os = "windows"))]
            command.args(["-c", &cmd_string]);

            // Inject context variables
            for (curr, val) in envs {
                command.env(curr, val);
            }

            match command.output() {
                Ok(output) => {
                    if !output.status.success() {
                        let stderr = String::from_utf8_lossy(&output.stderr);
                        eprintln!("[Hook Error] Command '{}' failed: {}", cmd_string, stderr);
                    }
                }
                Err(e) => {
                    eprintln!("[Hook Failed] Could not execute '{}': {}", cmd_string, e);
                }
            }
        });
    }
}
