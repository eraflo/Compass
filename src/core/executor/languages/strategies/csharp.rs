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
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;
use uuid::Uuid;

pub struct CSharpHandler;

impl LanguageDefinition for CSharpHandler {
    fn get_required_command(&self) -> &str {
        "dotnet"
    }

    fn prepare(&self, code: &str, temp_dir: &Path) -> Result<PathBuf> {
        // Create a unique directory for the project
        let id = Uuid::new_v4();
        let project_dir = temp_dir.join(format!("compass_cs_{}", id));
        std::fs::create_dir_all(&project_dir)?;

        // Run 'dotnet new console' to generate a valid .csproj for the user's SDK
        // We do this synchronously as prepare is running in a background thread
        let output = Command::new("dotnet")
            .args(["new", "console", "--force"])
            .current_dir(&project_dir)
            .output()
            .context("Failed to create .NET console project")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!(
                "Failed to initialize C# project: {}",
                stderr
            ));
        }

        // Overwrite Program.cs with user code
        // Modern .NET supports top-level statements, so we can dump the code directly
        let program_path = project_dir.join("Program.cs");
        std::fs::write(&program_path, code)
            .with_context(|| format!("Failed to write C# code to {}", program_path.display()))?;

        Ok(project_dir)
    }

    fn get_run_command(&self, prepared_path: &Path) -> Vec<String> {
        let path_str = prepared_path.to_string_lossy();
        vec![
            "dotnet".to_string(),
            "run".to_string(),
            "--project".to_string(),
            path_str.into_owned(),
            "--verbosity".to_string(),
            "quiet".to_string(),
            "--nologo".to_string(),
        ]
    }

    fn get_env_vars(&self) -> HashMap<String, String> {
        let mut vars = HashMap::new();
        vars.insert("CI".to_string(), "true".to_string());
        vars.insert("DOTNET_NOLOGO".to_string(), "true".to_string());
        vars.insert(
            "DOTNET_CLI_TELEMETRY_OPTOUT".to_string(),
            "true".to_string(),
        );
        vars
    }

    fn get_dangerous_patterns(&self) -> &[&'static str] {
        &[
            "System.Diagnostics.Process",
            "File.Delete",
            "Directory.Delete",
            "File.Move",
            "WebClient", // Can be used for download/exec
            "HttpClient",
        ]
    }

    fn get_extension(&self) -> &str {
        "cs"
    }
}
