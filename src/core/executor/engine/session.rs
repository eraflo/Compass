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
use crate::core::executor::languages::get_language_handler;
use crate::core::models::StepStatus;
use portable_pty::{CommandBuilder, PtySize, native_pty_system};
use std::io::Read;
use std::sync::mpsc::Sender;

/// Manages a PTY session for executing a shell command.
pub struct ShellSession {
    context: ExecutionContext,
}

impl ShellSession {
    /// Creates a new `ShellSession` with the given context.
    #[must_use]
    pub const fn new(context: ExecutionContext) -> Self {
        Self { context }
    }

    /// Executing via PTY and streaming output to a sender.
    pub fn run(
        &self,
        cmd_content: &str,
        language: Option<&str>,
        tx: &Sender<String>,
    ) -> StepStatus {
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

        // Prepare using Strategy
        let handler = get_language_handler(language);
        let temp_dir = std::env::temp_dir();

        let prepared_path = match handler.prepare(cmd_content, &temp_dir) {
            Ok(path) => path,
            Err(e) => {
                let _ = tx.send(format!("Failed to prepare code: {e}\n"));
                return StepStatus::Failed;
            }
        };

        let run_cmd = handler.get_run_command(&prepared_path);
        let mut cmd = CommandBuilder::new(&run_cmd[0]);
        for arg in &run_cmd[1..] {
            cmd.arg(arg);
        }

        cmd.cwd(&self.context.current_dir);
        for (key, val) in &self.context.env_vars {
            cmd.env(key, val);
        }

        // Apply environment variables from the language strategy
        for (key, val) in handler.get_env_vars() {
            cmd.env(key, val);
        }

        // Spawn child
        let mut child = match pty_pair.slave.spawn_command(cmd) {
            Ok(child) => child,
            Err(e) => {
                let _ = tx.send(format!("Error spawning process: {e}\n"));
                // Try to cleanup
                let _ = std::fs::remove_file(&prepared_path);
                return StepStatus::Failed;
            }
        };

        // Drop slave now - child has it
        drop(pty_pair.slave);

        // Streaming output via a dedicated thread to avoid blocking wait()
        let mut reader = match pty_pair.master.try_clone_reader() {
            Ok(r) => r,
            Err(e) => {
                let _ = tx.send(format!("Error getting reader: {e}\n"));
                return StepStatus::Failed;
            }
        };

        let tx_output = tx.clone();
        let read_thread = std::thread::spawn(move || {
            let mut buffer = [0u8; 4096]; // Larger buffer
            while let Ok(n) = reader.read(&mut buffer) {
                if n == 0 {
                    break;
                }
                if let Ok(text) = std::str::from_utf8(&buffer[..n]) {
                    // Send to channel
                    let _ = tx_output.send(text.to_string());
                }
            }
        });

        // Wait for child to finish
        let status = child.wait();

        // Cleanup temporary file
        let _ = std::fs::remove_file(&prepared_path);

        // On Windows, give ConPTY a tiny bit of time to flush
        if cfg!(target_os = "windows") {
            std::thread::sleep(std::time::Duration::from_millis(50));
        }

        // Explicitly drop master after child finishes to signal EOF to reader thread
        drop(pty_pair.master);

        // Join reader thread to ensure all output is forwarded
        let _ = read_thread.join();

        status.map_or(StepStatus::Failed, |s| {
            if s.success() {
                StepStatus::Success
            } else {
                StepStatus::Failed
            }
        })
    }
}
