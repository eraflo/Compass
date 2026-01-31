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

pub struct ShellHandler {
    lang: String,
}

impl ShellHandler {
    pub fn new(lang: &str) -> Self {
        Self {
            lang: lang.to_string(),
        }
    }

    fn is_powershell(&self) -> bool {
        self.lang == "powershell" || (self.lang == "default" && cfg!(target_os = "windows"))
    }

    fn is_cmd(&self) -> bool {
        self.lang == "cmd" || (self.lang == "batch")
    }
}

impl LanguageDefinition for ShellHandler {
    fn get_required_command(&self) -> &str {
        if self.is_powershell() {
            "powershell"
        } else if self.is_cmd() {
            "cmd"
        } else if self.lang == "bash" || self.lang == "zsh" {
            // explicit bash/zsh
            if self.lang == "zsh" { "zsh" } else { "bash" }
        } else if self.lang == "sh" {
            "sh"
        } else if self.lang == "default" {
            // Unix default
            "sh"
        } else {
            // Fallback for unknown shells to sh or bash?
            "bash"
        }
    }

    fn prepare(&self, code: &str, temp_dir: &Path) -> Result<PathBuf> {
        let extension = if self.is_powershell() {
            "ps1"
        } else if self.is_cmd() {
            "bat"
        } else {
            "sh"
        };

        let filename = format!("script_{}.{}", Uuid::new_v4(), extension);
        let file_path = temp_dir.join(filename);

        std::fs::write(&file_path, code)
            .with_context(|| format!("Failed to write shell script to {}", file_path.display()))?;

        Ok(file_path)
    }

    fn get_run_command(&self, prepared_path: &Path) -> Vec<String> {
        let cmd = self.get_required_command().to_string();

        if self.is_powershell() {
            // Use -ExecutionPolicy Bypass to ensure the script runs despite local restrictions
            vec![
                cmd,
                "-ExecutionPolicy".to_string(),
                "Bypass".to_string(),
                "-File".to_string(),
                prepared_path.to_string_lossy().to_string(),
            ]
        } else if self.is_cmd() {
            vec![
                cmd,
                "/C".to_string(),
                prepared_path.to_string_lossy().to_string(),
            ]
        } else {
            // sh /tmp/script.sh
            // On Windows, bash/sh often requires forward slashes
            let path_str = prepared_path.to_string_lossy().to_string();
            let path_argument = if cfg!(target_os = "windows") {
                path_str.replace('\\', "/")
            } else {
                path_str
            };
            vec![cmd, path_argument]
        }
    }

    fn get_dangerous_patterns(&self) -> &[&'static str] {
        &[
            "rm -rf /",
            "rm -rf *",
            "mkfs",
            "> /dev/sd",
            "dd if=",
            ":(){:|:&};:", // Fork bomb
            "mv /",
            "chmod -R 777 /",
        ]
    }

    fn get_extension(&self) -> &str {
        if self.is_powershell() {
            "ps1"
        } else if self.is_cmd() {
            "bat"
        } else {
            "sh"
        }
    }
}
