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
use crate::core::models::{Step, StepStatus};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::sync::Mutex;

#[derive(Serialize, Deserialize, Debug)]
struct RpcRequest {
    jsonrpc: String,
    method: String,
    params: Option<serde_json::Value>,
    id: Option<u64>,
}

#[derive(Serialize, Deserialize, Debug)]
struct RpcResponse {
    jsonrpc: String,
    result: Option<serde_json::Value>,
    error: Option<RpcError>,
    id: Option<u64>,
}

#[derive(Serialize, Deserialize, Debug)]
struct RpcError {
    code: i32,
    message: String,
}

struct HeadlessState {
    steps: Vec<Step>,
    executor: Executor,
}

pub async fn start_headless_server(
    steps: Vec<Step>,
    path: PathBuf,
    sandbox: bool,
    image: String,
) -> anyhow::Result<()> {
    let mut executor = Executor::new();
    // Default CWD to the parent of the README file
    executor.context.current_dir = if path.is_file() {
        path.parent().unwrap_or(&path).to_path_buf()
    } else {
        path.clone()
    };
    executor.context.sandbox_enabled = sandbox;
    executor.context.docker_image = image;

    let state = Arc::new(Mutex::new(HeadlessState { steps, executor }));

    let stdin = tokio::io::stdin();
    let mut reader = BufReader::new(stdin);
    let mut line = String::new();

    loop {
        line.clear();
        let bytes_read = reader.read_line(&mut line).await?;
        if bytes_read == 0 {
            break; // EOF
        }

        let req_str = line.trim();
        if req_str.is_empty() {
            continue;
        }

        let req: RpcRequest = match serde_json::from_str(req_str) {
            Ok(r) => r,
            Err(e) => {
                send_error(None, -32700, &format!("Parse error: {}", e)).await;
                continue;
            }
        };

        let state_clone = state.clone();

        // Process request
        match req.method.as_str() {
            "get_steps" => {
                let state = state_clone.lock().await;
                send_response(req.id, serde_json::to_value(&state.steps)?).await;
            }
            "execute_step" => {
                if let Some(params) = req.params {
                    let index = params
                        .get("index")
                        .and_then(|v| v.as_u64())
                        .map(|v| v as usize);
                    if let Some(idx) = index {
                        let mut state = state_clone.lock().await; // Lock for duration of execution
                        if idx < state.steps.len() {
                            let mut final_status = StepStatus::Success;
                            let mut full_output = String::new();
                            let (tx, rx): (
                                std::sync::mpsc::Sender<String>,
                                std::sync::mpsc::Receiver<String>,
                            ) = std::sync::mpsc::channel();

                            // Spawn a thread to stream logs as JSON-RPC notifications
                            let logger_handle = std::thread::spawn(move || {
                                let mut collected = String::new();
                                while let Ok(msg) = rx.recv() {
                                    collected.push_str(&msg);
                                    // Send "log" notification
                                    let note = RpcRequest {
                                        jsonrpc: "2.0".to_string(),
                                        method: "log".to_string(),
                                        params: Some(serde_json::json!({ "output": msg })),
                                        id: None,
                                    };
                                    if let Ok(json) = serde_json::to_string(&note) {
                                        println!("{}", json);
                                    }
                                }
                                collected
                            });

                            // Clone needed blocks to avoid borrowing conflict with state
                            let code_blocks = state.steps[idx].code_blocks.clone();

                            for block in code_blocks {
                                let status = state.executor.execute_streamed(
                                    &block.content,
                                    block.language.as_deref(),
                                    true, // Headless assumes intention to run
                                    &tx,
                                );
                                if status != StepStatus::Success {
                                    final_status = status;
                                    break;
                                }
                            }

                            // Close channel to stop logger
                            drop(tx);

                            // Wait for logger and get valid full output
                            if let Ok(collected_output) = logger_handle.join() {
                                full_output = collected_output;
                            }

                            state.steps[idx].status = final_status;
                            if !full_output.is_empty() {
                                state.steps[idx].output = full_output.clone();
                            }

                            send_response(
                                req.id,
                                serde_json::json!({
                                   "status": final_status,
                                   "output": state.steps[idx].output
                                }),
                            )
                            .await;
                        } else {
                            send_error(req.id, -32602, "Invalid params: index out of bounds").await;
                        }
                    } else {
                        send_error(req.id, -32602, "Invalid params: missing index").await;
                    }
                }
            }
            _ => {
                send_error(req.id, -32601, "Method not found").await;
            }
        }
    }

    Ok(())
}

async fn send_response(id: Option<u64>, result: Value) {
    let resp = RpcResponse {
        jsonrpc: "2.0".to_string(),
        result: Some(result),
        error: None,
        id,
    };
    if let Ok(json) = serde_json::to_string(&resp) {
        let mut stdout = tokio::io::stdout();
        let _ = stdout.write_all(json.as_bytes()).await;
        let _ = stdout.write_all(b"\n").await;
        let _ = stdout.flush().await;
    }
}

async fn send_error(id: Option<u64>, code: i32, message: &str) {
    let resp = RpcResponse {
        jsonrpc: "2.0".to_string(),
        result: None,
        error: Some(RpcError {
            code,
            message: message.to_string(),
        }),
        id,
    };
    if let Ok(json) = serde_json::to_string(&resp) {
        let mut stdout = tokio::io::stdout();
        let _ = stdout.write_all(json.as_bytes()).await;
        let _ = stdout.write_all(b"\n").await;
        let _ = stdout.flush().await;
    }
}
