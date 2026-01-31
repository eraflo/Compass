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

use crate::core::executor::languages::get_language_handler;
use crate::core::models::Step;
use std::collections::HashSet;
use which::which;

/// Result of the dependency check.
#[derive(Debug)]
pub struct CheckResult {
    /// List of commands found in the system.
    pub present: Vec<String>,
    /// List of commands missing from the system.
    pub missing: Vec<String>,
}

/// Scans the provided steps for potential external dependencies (commands)
/// and verifies if they exist on the host system.
///
/// This uses a heuristic approach to identify commands in shell code blocks.
pub fn check_dependencies(steps: &[Step]) -> CheckResult {
    let mut candidates = HashSet::new();
    let builtins = get_builtins();

    for step in steps {
        for block in &step.code_blocks {
            // Only check shell-like blocks or blocks with no language specified
            let is_shell = block.language.as_ref().is_none_or(|lang| {
                ["bash", "sh", "shell", "zsh", "fish", "cmd", "powershell"].contains(&lang.as_str())
            });

            if !is_shell {
                 if let Some(lang) = &block.language {
                    let handler = get_language_handler(Some(lang));
                    let cmd = handler.get_required_command();
                    // Filter out fallback shells (sh, powershell, cmd) returned by the default handler
                    // when the language is not explicitly supported. We only want to report
                    // missing dependencies for specific required tools (e.g. "go", "python", "cargo").
                    if cmd != "sh" && cmd != "powershell" && cmd != "cmd" {
                        candidates.insert(cmd.to_string());
                    }
                }
                continue;
            }

            for line in block.content.lines() {
                let line = line.trim();

                // Skip comments and common non-command lines
                if line.is_empty() || line.starts_with('#') {
                    continue;
                }

                // Handle simple command chains (&&, ||, ;, |)
                // This is a naive split to catch more commands
                let parts: Vec<&str> = line.split(['&', '|', ';']).collect();

                for part in parts {
                    let part = part.trim();
                    if part.is_empty() {
                        continue;
                    }

                    // Extract first word
                    if let Some(cmd) = part.split_whitespace().next() {
                        // Skip variable assignments (VAR=val)
                        if cmd.contains('=') {
                            continue;
                        }

                        // Skip sudo (check the next word)
                        let target_cmd = if cmd == "sudo" {
                            part.split_whitespace().nth(1).unwrap_or("")
                        } else {
                            cmd
                        };

                        if target_cmd.is_empty() {
                            continue;
                        }

                        // Filter out paths (./script), builtins, and variables ($VAR)
                        if !target_cmd.contains('/')
                            && !target_cmd.contains('\\')
                            && !target_cmd.starts_with('$')
                            && !builtins.contains(target_cmd)
                        {
                            candidates.insert(target_cmd.to_string());
                        }
                    }
                }
            }
        }
    }

    let mut missing = Vec::new();
    let mut present = Vec::new();

    for cmd in candidates {
        if which(&cmd).is_ok() {
            present.push(cmd);
        } else {
            missing.push(cmd);
        }
    }

    present.sort();
    missing.sort();

    CheckResult { present, missing }
}

fn get_builtins() -> HashSet<&'static str> {
    HashSet::from([
        // Shell builtins & common utils usually present
        "cd", "echo", "printf", "export", "unset", "set", "alias", "unalias", "source", ".", "eval",
        "exec", "exit", "return", "true", "false", "test", "[", "[[", "read", "wait", "bg", "fg",
        "jobs", "kill", "history", "pwd", "pushd", "popd", "dirs", "shift", "umask", "if", "then",
        "else", "elif", "fi", "for", "while", "until", "do", "done", "case", "esac", "function",
        "select", "break",
        "continue",
        // Common standard utilities (often expected to assume present, but debatable)
        // Leaving these out of builtins means we WILL check for them,
        // which is good: a fresh container might miss 'curl' or 'git'.
        // So we strictly keep to shell syntax keywords.
    ])
}
