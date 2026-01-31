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
use portable_pty::{CommandBuilder, PtySize, native_pty_system};
use std::collections::HashMap;
use std::io::Read;
use std::path::PathBuf;
use std::sync::mpsc::Sender;

/// Holds the mutable state of the execution environment.
#[derive(Debug, Clone, Default)]
pub struct ExecutionContext {
    pub current_dir: PathBuf,
    pub env_vars: HashMap<String, String>,
}

impl ExecutionContext {
    #[must_use]
    pub fn new() -> Self {
        let current_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        Self {
            current_dir,
            env_vars: HashMap::new(),
        }
    }
}

/// Handles "built-in" commands that affect the `ExecutionContext` directly.
pub struct BuiltinHandler;

impl BuiltinHandler {
    /// Scans the content for built-in patterns and updates the context.
    pub fn pre_process(cmd_content: &str, context: &mut ExecutionContext) {
        for line in cmd_content.lines() {
            let trimmed = line.trim();

            // Detect 'cd'
            if let Some(rest) = trimmed.strip_prefix("cd ") {
                let path_str = rest.trim().trim_matches(|c| c == '\"' || c == '\'');
                let new_path = context.current_dir.join(path_str);
                if new_path.exists() && new_path.is_dir() {
                    context.current_dir = new_path.canonicalize().unwrap_or(new_path);
                }
            }

            // Detect 'export'
            if let Some(rest) = trimmed.strip_prefix("export ") {
                let assignment = rest.trim();
                if let Some((key, val)) = assignment.split_once('=') {
                    let val = val.trim_matches(|c| c == '\"' || c == '\'');
                    context
                        .env_vars
                        .insert(key.trim().to_string(), val.to_string());
                }
            }
        }
    }
}

/// Manages a PTY session for executing a shell command.
pub struct ShellSession {
    context: ExecutionContext,
}

impl ShellSession {
    #[must_use]
    pub const fn new(context: ExecutionContext) -> Self {
        Self { context }
    }

    /// Executing via PTY and streaming output to a sender.
    pub fn run(&self, cmd_content: &str, tx: &Sender<String>) -> StepStatus {
        let pty_system = native_pty_system();
        let pty_pair = match pty_system.openpty(PtySize {
            rows: 24,
            cols: 80,
            pixel_width: 0,
            pixel_height: 0,
        }) {
            Ok(pair) => pair,
            Err(e) => {
                let _ = tx.send(format!("Error opening PTY: {e}\n"));
                return StepStatus::Failed;
            }
        };

        // Command selection based on OS
        let mut cmd = if cfg!(target_os = "windows") {
            let mut c = CommandBuilder::new("powershell");
            c.arg("-Command");
            c.arg(cmd_content);
            c
        } else {
            let mut c = CommandBuilder::new("sh");
            c.arg("-c");
            c.arg(cmd_content);
            c
        };

        cmd.cwd(&self.context.current_dir);
        for (key, val) in &self.context.env_vars {
            cmd.env(key, val);
        }

        // Spawn child
        let mut child = match pty_pair.slave.spawn_command(cmd) {
            Ok(child) => child,
            Err(e) => {
                let _ = tx.send(format!("Error spawning process: {e}\n"));
                return StepStatus::Failed;
            }
        };

        // Slave is no longer needed after spawn
        drop(pty_pair.slave);

        // Streaming output
        let mut reader = match pty_pair.master.try_clone_reader() {
            Ok(r) => r,
            Err(e) => {
                let _ = tx.send(format!("Error getting reader: {e}\n"));
                return StepStatus::Failed;
            }
        };

        let mut buffer = [0u8; 1024];
        loop {
            match reader.read(&mut buffer) {
                Ok(0) => break, // EOF
                Ok(n) => {
                    let output = String::from_utf8_lossy(&buffer[..n]).to_string();
                    if tx.send(output).is_err() {
                        break;
                    }
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::Interrupted => {}
                Err(_) => break,
            }
        }

        // Wait for exit
        child.wait().map_or(StepStatus::Failed, |status| {
            if status.success() {
                StepStatus::Success
            } else {
                StepStatus::Failed
            }
        })
    }
}

/// The main entry point for the execution engine.
pub struct Executor {
    pub context: ExecutionContext,
}

impl Executor {
    #[must_use]
    pub fn new() -> Self {
        Self {
            context: ExecutionContext::new(),
        }
    }

    /// Orchestrates the execution of a code block.
    pub fn execute_streamed(&mut self, cmd_content: &str, tx: &Sender<String>) -> StepStatus {
        // 1. Handle side-effects (builtins)
        BuiltinHandler::pre_process(cmd_content, &mut self.context);

        // 2. Run via ShellSession
        let session = ShellSession::new(self.context.clone());
        session.run(cmd_content, tx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builtin_export() {
        let mut ctx = ExecutionContext::default();
        BuiltinHandler::pre_process("export FOO=BAR\nexport BAZ=\"QUX\"", &mut ctx);
        assert_eq!(ctx.env_vars.get("FOO").unwrap(), "BAR");
        assert_eq!(ctx.env_vars.get("BAZ").unwrap(), "QUX");
    }

    #[test]
    fn test_builtin_cd_not_matching() {
        let mut ctx = ExecutionContext {
            current_dir: PathBuf::from("."),
            env_vars: HashMap::new(),
        };
        let initial_dir = ctx.current_dir.clone();
        BuiltinHandler::pre_process("nonexistent_command", &mut ctx);
        assert_eq!(ctx.current_dir, initial_dir);
    }
}
