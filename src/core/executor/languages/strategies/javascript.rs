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

pub struct JsHandler;

impl LanguageDefinition for JsHandler {
    fn get_required_command(&self) -> &str {
        "node"
    }

    fn prepare(&self, code: &str, temp_dir: &Path) -> Result<PathBuf> {
        let filename = format!("script_{}.js", Uuid::new_v4());
        let file_path = temp_dir.join(filename);
        std::fs::write(&file_path, code)
            .with_context(|| format!("Failed to write JS script to {}", file_path.display()))?;
        Ok(file_path)
    }

    fn get_run_command(&self, prepared_path: &Path) -> Vec<String> {
        vec![
            "node".to_string(),
            prepared_path.to_string_lossy().to_string(),
        ]
    }

    fn get_dangerous_patterns(&self) -> &[&'static str] {
        &[
            "child_process",
            "exec(",
            "spawn(",
            "fs.rm",
            "fs.unlink",
            "fs.writeFile",
            "process.kill",
        ]
    }

    fn get_extension(&self) -> &str {
        "js"
    }
}
