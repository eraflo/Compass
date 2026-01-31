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

use crate::core::executor::Executor;
use crate::ui::state::ExecutionMessage;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;

/// Manages background execution of commands.
pub struct ExecutionManager {
    /// The core executor logic (holds context).
    /// Wrapped in a specialized way if needed, but here we clone context for threads.
    pub executor: Executor,
    /// Channel for receiving execution updates in the main thread.
    rx: Receiver<ExecutionMessage>,
    /// Sender to be cloned for background threads.
    tx: Sender<ExecutionMessage>,
}

impl ExecutionManager {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel();
        Self {
            executor: Executor::new(),
            rx,
            tx,
        }
    }

    /// Spawns a background thread to execute the given content.
    pub fn execute_background(
        &self,
        index: usize,
        content: String,
        language: Option<String>,
        bypass_safety: bool,
    ) {
        let tx = self.tx.clone();
        let current_dir = self.executor.context.current_dir.clone();
        let env_vars = self.executor.context.env_vars.clone();

        thread::spawn(move || {
            let mut local_executor = Executor {
                context: crate::core::executor::ExecutionContext {
                    current_dir,
                    env_vars,
                },
            };
            let (stream_tx, stream_rx) = mpsc::channel::<String>();

            let tx_for_streaming = tx.clone();

            // Spawn a sub-thread to forward streaming output
            thread::spawn(move || {
                while let Ok(partial) = stream_rx.recv() {
                    tx_for_streaming
                        .send(ExecutionMessage::OutputPartial(index, partial))
                        .unwrap();
                }
            });

            // Execute the command
            let status = local_executor.execute_streamed(
                &content,
                language.as_deref(),
                bypass_safety,
                &stream_tx,
            );

            // Send finish event
            tx.send(ExecutionMessage::Finished(
                index,
                status,
                local_executor.context.current_dir,
                local_executor.context.env_vars,
            ))
            .unwrap();
        });
    }

    /// Polls for any new execution messages.
    pub fn poll_messages(&self) -> Vec<ExecutionMessage> {
        let mut messages = Vec::new();
        while let Ok(msg) = self.rx.try_recv() {
            messages.push(msg);
        }
        messages
    }
}
