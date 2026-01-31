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

use crate::core::export::models::ExportReport;
use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

/// Exports the report to a JSON file.
pub fn export(report: &ExportReport, output_path: &Path) -> Result<PathBuf> {
    let content =
        serde_json::to_string_pretty(report).context("Failed to serialize report to JSON")?;

    // Ensure parent directory exists
    #[allow(clippy::collapsible_if)]
    if let Some(parent) = output_path.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
        }
    }

    fs::write(output_path, content)
        .with_context(|| format!("Failed to write JSON report to: {}", output_path.display()))?;

    Ok(output_path.to_path_buf())
}
