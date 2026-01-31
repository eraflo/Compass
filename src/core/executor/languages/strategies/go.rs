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

pub struct GoHandler;

impl LanguageDefinition for GoHandler {
    fn get_required_command(&self) -> &str {
        "go"
    }

    fn prepare(&self, code: &str, temp_dir: &Path) -> Result<PathBuf> {
        let filename = format!("main_{}.go", Uuid::new_v4());
        let file_path = temp_dir.join(filename);

        // Go needs a package main to run
        let content = if !code.contains("package main") {
            format!("package main\n\n{}", code)
        } else {
            code.to_string()
        };

        std::fs::write(&file_path, content)
            .with_context(|| format!("Failed to write Go script to {}", file_path.display()))?;
        Ok(file_path)
    }

    fn get_run_command(&self, prepared_path: &Path) -> Vec<String> {
        vec![
            "go".to_string(),
            "run".to_string(),
            prepared_path.to_string_lossy().to_string(),
        ]
    }

    fn get_dangerous_patterns(&self) -> &[&'static str] {
        &["os/exec", "os.Remove", "syscall.Exec", "os.RemoveAll"]
    }

    fn get_extension(&self) -> &str {
        "go"
    }
}
