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

use std::collections::HashMap;
use std::path::PathBuf;

/// Holds the mutable state of the execution environment.
#[derive(Debug, Clone, Default)]
pub struct ExecutionContext {
    pub current_dir: PathBuf,
    pub env_vars: HashMap<String, String>,
    pub sandbox_enabled: bool,
    pub docker_image: String,
}

impl ExecutionContext {
    /// Creates a new `ExecutionContext` with the system's current working directory.
    #[must_use]
    pub fn new() -> Self {
        let current_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        Self {
            current_dir,
            env_vars: HashMap::new(),
            sandbox_enabled: false,
            docker_image: "ubuntu:latest".to_string(),
        }
    }
}
