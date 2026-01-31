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
use minijinja::Environment;
use std::fs;
use std::path::{Path, PathBuf};

/// The Markdown template used for report generation.
const MARKDOWN_TEMPLATE: &str = include_str!("../../../../templates/report.md");

/// Exports the report to a Markdown file.
pub fn export(report: &ExportReport, output_path: &Path) -> Result<PathBuf> {
    let mut env = Environment::new();
    env.add_template("report.md", MARKDOWN_TEMPLATE)
        .context("Failed to load markdown template")?;

    let template = env
        .get_template("report.md")
        .context("Failed to get markdown template")?;

    let rendered = template
        .render(report)
        .context("Failed to render markdown report")?;

    // Ensure parent directory exists
    #[allow(clippy::collapsible_if)]
    if let Some(parent) = output_path.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
        }
    }

    fs::write(output_path, rendered).with_context(|| {
        format!(
            "Failed to write Markdown report to: {}",
            output_path.display()
        )
    })?;

    Ok(output_path.to_path_buf())
}
