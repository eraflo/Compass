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

use super::context::ExecutionContext;

/// Handles "built-in" commands that affect the `ExecutionContext` directly.
pub struct BuiltinHandler;

impl BuiltinHandler {
    /// Scans the content for built-in patterns, updates the context, and returns:
    /// 1. The remaining content that should actually be executed by the shell.
    /// 2. A simulated output string for the commands handled exclusively.
    pub fn process(cmd_content: &str, context: &mut ExecutionContext) -> (String, String) {
        let mut remaining_lines = Vec::new();
        let mut simulated_output = String::new();

        for line in cmd_content.lines() {
            let trimmed = line.trim();
            let mut handled_exclusively = false;

            // Detect 'cd'
            if let Some(rest) = trimmed.strip_prefix("cd ") {
                let path_str = rest.trim().trim_matches(|c| c == '\"' || c == '\'');
                let new_path = context.current_dir.join(path_str);
                if new_path.exists() && new_path.is_dir() {
                    let mut final_path = new_path.canonicalize().unwrap_or(new_path);

                    // On Windows, canonicalize() adds the \\?\ prefix.
                    // This can break some tools, so we strip it.
                    if cfg!(target_os = "windows") {
                        let path_str = final_path.to_string_lossy();
                        if let Some(stripped) = path_str.strip_prefix(r"\\?\") {
                            final_path = std::path::PathBuf::from(stripped);
                        }
                    }

                    context.current_dir = final_path;
                    simulated_output.push_str(&format!(
                        "cd: {} (Handled by Compass)\n",
                        context.current_dir.display()
                    ));
                }
                handled_exclusively = true;
            }

            // Detect 'export'
            if let Some(rest) = trimmed.strip_prefix("export ") {
                let assignment = rest.trim();
                if let Some((key, val)) = assignment.split_once('=') {
                    let val = val.trim_matches(|c| c == '\"' || c == '\'');
                    context
                        .env_vars
                        .insert(key.trim().to_string(), val.to_string());

                    simulated_output.push_str(&format!(
                        "export: {}={} (Handled by Compass)\n",
                        key.trim(),
                        val
                    ));
                }
                handled_exclusively = true;
            }

            if !handled_exclusively {
                remaining_lines.push(line);
            }
        }

        (remaining_lines.join("\n"), simulated_output)
    }
}
