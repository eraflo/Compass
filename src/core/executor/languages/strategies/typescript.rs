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

use crate::core::executor::languages::definition::LanguageDefinition;
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use uuid::Uuid;

pub struct TsHandler;

impl LanguageDefinition for TsHandler {
    fn get_required_command(&self) -> &str {
        // We assume ts-node is available, or we could fallback to node + tsc but that's complex.
        // For a script runner, ts-node (or deno) is standard.
        if cfg!(target_os = "windows") {
            "ts-node.cmd"
        } else {
            "ts-node"
        }
    }

    fn prepare(&self, code: &str, temp_dir: &Path) -> Result<PathBuf> {
        let filename = format!("script_{}.ts", Uuid::new_v4());
        let file_path = temp_dir.join(filename);
        std::fs::write(&file_path, code)
            .with_context(|| format!("Failed to write TS script to {}", file_path.display()))?;
        Ok(file_path)
    }

    fn get_run_command(&self, prepared_path: &Path) -> Vec<String> {
        vec![
            self.get_required_command().to_string(),
            prepared_path.to_string_lossy().to_string(),
        ]
    }

    fn get_dangerous_patterns(&self) -> &[&'static str] {
        &["child_process", "exec(", "Deno.run", "fs.rm", "fs.unlink"]
    }

    fn get_extension(&self) -> &str {
        "ts"
    }
}
