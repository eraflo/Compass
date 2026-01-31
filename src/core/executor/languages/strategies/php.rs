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

pub struct PhpHandler;

impl LanguageDefinition for PhpHandler {
    fn get_required_command(&self) -> &str {
        "php"
    }

    fn prepare(&self, code: &str, temp_dir: &Path) -> Result<PathBuf> {
        let filename = format!("script_{}.php", Uuid::new_v4());
        let file_path = temp_dir.join(filename);

        let content = if !code.starts_with("<?php") {
            format!("<?php\n{}", code)
        } else {
            code.to_string()
        };

        std::fs::write(&file_path, content)
            .with_context(|| format!("Failed to write PHP script to {}", file_path.display()))?;
        Ok(file_path)
    }

    fn get_run_command(&self, prepared_path: &Path) -> Vec<String> {
        vec![
            "php".to_string(),
            prepared_path.to_string_lossy().to_string(),
        ]
    }

    fn get_dangerous_patterns(&self) -> &[&'static str] {
        &[
            "exec(",
            "shell_exec",
            "system(",
            "passthru",
            "proc_open",
            "unlink(",
        ]
    }

    fn get_extension(&self) -> &str {
        "php"
    }
}
