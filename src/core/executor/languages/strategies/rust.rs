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

pub struct RustHandler;

impl LanguageDefinition for RustHandler {
    fn get_required_command(&self) -> &str {
        "rustc"
    }

    fn prepare(&self, code: &str, temp_dir: &Path) -> Result<PathBuf> {
        let filename = format!("script_{}.rs", Uuid::new_v4());
        let file_path = temp_dir.join(filename);

        let content = if !code.contains("fn main") {
            format!("fn main() {{\n{}\n}}", code)
        } else {
            code.to_string()
        };

        std::fs::write(&file_path, content)
            .with_context(|| format!("Failed to write Rust script to {}", file_path.display()))?;
        Ok(file_path)
    }

    fn get_run_command(&self, prepared_path: &Path) -> Vec<String> {
        // Compile and run immediately using a shell wrapper.
        // Rust requires a build step, so we chain `rustc` and the execution of the binary.
        let output_name = prepared_path.with_extension("exe");
        let output_str = output_name.to_string_lossy();
        let source_str = prepared_path.to_string_lossy();

        if cfg!(target_os = "windows") {
            vec![
                "powershell".to_string(),
                "-Command".to_string(),
                format!(
                    "rustc \"{}\" -o \"{}\"; if ($?) {{ & \"{}\" }}",
                    source_str, output_str, output_str
                ),
            ]
        } else {
            vec![
                "sh".to_string(),
                "-c".to_string(),
                format!(
                    "rustc \"{}\" -o \"{}\" && \"{}\"",
                    source_str, output_str, output_str
                ),
            ]
        }
    }

    fn get_dangerous_patterns(&self) -> &[&'static str] {
        &["std::process", "std::fs::remove", "Command::new"]
    }

    fn get_extension(&self) -> &str {
        "rs"
    }
}
