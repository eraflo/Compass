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

//! # Export Module
//!
//! This module provides functionality to export the current Compass session
//! into various formats (JSON and Markdown). This is essential for debugging,
//! sharing session results, and onboarding support.
//!
//! ## Extensibility
//!
//! New export formats can be added by creating a new module in `formats/`
//! and calling it from the `Exporter` struct.

pub mod formats;
pub mod models;

use crate::core::models::{Step, StepStatus};
use anyhow::Result;
use chrono::{Local, Utc};
use models::{
    EnvironmentInfo, ExportReport, ExportedCodeBlock, ExportedStep, ReportMetadata, ReportSummary,
};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Exports session data to various formats.
pub struct Exporter;

impl Exporter {
    /// Generates an export report from the current session state.
    ///
    /// # Arguments
    ///
    /// * `steps` - The list of steps with their current state.
    /// * `readme_path` - The path to the README file.
    /// * `current_dir` - The current working directory.
    /// * `env_vars` - Environment variables set during the session.
    /// * `placeholders` - Placeholder values provided by the user.
    /// * `version` - The Compass version string.
    ///
    /// # Returns
    ///
    /// An `ExportReport` containing all session data.
    #[must_use]
    pub fn generate_report(
        steps: &[Step],
        readme_path: &Path,
        current_dir: &Path,
        env_vars: &HashMap<String, String>,
        placeholders: &HashMap<String, String>,
        version: &str,
    ) -> ExportReport {
        // Convert steps to exportable format
        let exported_steps: Vec<ExportedStep> = steps
            .iter()
            .enumerate()
            .map(|(i, step)| ExportedStep {
                number: i + 1,
                title: step.title.clone(),
                description: step.description.clone(),
                status: Self::status_to_string(step.status),
                code_blocks: step
                    .code_blocks
                    .iter()
                    .map(|b| ExportedCodeBlock {
                        language: b.language.clone(),
                        content: b.content.clone(),
                    })
                    .collect(),
                output: step.output.clone(),
            })
            .collect();

        // Calculate summary statistics (only executable steps)
        let executable_steps: Vec<&Step> = steps.iter().filter(|s| s.is_executable()).collect();
        let total_steps = executable_steps.len();
        let completed_steps = executable_steps
            .iter()
            .filter(|s| s.status == StepStatus::Success)
            .count();
        let failed_steps = executable_steps
            .iter()
            .filter(|s| s.status == StepStatus::Failed)
            .count();
        let running_steps = executable_steps
            .iter()
            .filter(|s| s.status == StepStatus::Running)
            .count();
        let pending_steps = executable_steps
            .iter()
            .filter(|s| s.status == StepStatus::Pending)
            .count();

        #[allow(clippy::cast_precision_loss)]
        let completion_percentage = if total_steps > 0 {
            (completed_steps as f32 / total_steps as f32) * 100.0
        } else {
            0.0
        };

        ExportReport {
            metadata: ReportMetadata {
                compass_version: version.to_string(),
                generated_at: Utc::now().to_rfc3339(),
                generated_at_local: Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
                readme_path: readme_path.to_string_lossy().to_string(),
            },
            summary: ReportSummary {
                total_steps,
                completed_steps,
                failed_steps,
                pending_steps,
                running_steps,
                completion_percentage,
            },
            steps: exported_steps,
            environment: EnvironmentInfo {
                current_dir: current_dir.to_string_lossy().to_string(),
                env_vars: env_vars.clone(),
                placeholders: placeholders.clone(),
            },
        }
    }

    /// Converts a `StepStatus` to a human-readable string.
    fn status_to_string(status: StepStatus) -> String {
        match status {
            StepStatus::Pending => "⏳ Pending".to_string(),
            StepStatus::Running => "🔄 Running".to_string(),
            StepStatus::Success => "✅ Success".to_string(),
            StepStatus::Failed => "❌ Failed".to_string(),
            StepStatus::Skipped => "🚫 Skipped".to_string(),
        }
    }

    /// Exports the report to a JSON file.
    ///
    /// # Arguments
    ///
    /// * `report` - The report to export.
    /// * `output_path` - The path where the JSON file will be written.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be written or JSON serialization fails.
    pub fn export_json(report: &ExportReport, output_path: &Path) -> Result<PathBuf> {
        formats::json::export(report, output_path)
    }

    /// Exports the report to a Markdown file.
    ///
    /// This format is human-readable and can be shared easily via email,
    /// Slack, or other communication tools.
    ///
    /// # Arguments
    ///
    /// * `report` - The report to export.
    /// * `output_path` - The path where the Markdown file will be written.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be written.
    pub fn export_markdown(report: &ExportReport, output_path: &Path) -> Result<PathBuf> {
        formats::markdown::export(report, output_path)
    }

    /// Generates default output paths for the export files.
    ///
    /// The files are created in the current working directory with timestamped names.
    ///
    /// # Arguments
    ///
    /// * `base_dir` - The directory where files will be created.
    ///
    /// # Returns
    ///
    /// A tuple of (`json_path`, `markdown_path`).
    #[must_use]
    pub fn default_output_paths(base_dir: &Path) -> (PathBuf, PathBuf) {
        let timestamp = Local::now().format("%Y%m%d_%H%M%S");
        let json_path = base_dir.join(format!("compass-report_{timestamp}.json"));
        let md_path = base_dir.join(format!("compass-report_{timestamp}.md"));
        (json_path, md_path)
    }

    /// Exports to both JSON and Markdown formats.
    ///
    /// # Arguments
    ///
    /// * `report` - The report to export.
    /// * `base_dir` - The directory where files will be created.
    ///
    /// # Returns
    ///
    /// A tuple of the created file paths (`json_path`, `markdown_path`).
    ///
    /// # Errors
    ///
    /// Returns an error if any file cannot be written.
    pub fn export_both(report: &ExportReport, base_dir: &Path) -> Result<(PathBuf, PathBuf)> {
        let (json_path, md_path) = Self::default_output_paths(base_dir);

        let json_result = Self::export_json(report, &json_path)?;
        let md_result = Self::export_markdown(report, &md_path)?;

        Ok((json_result, md_result))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::models::CodeBlock;
    use std::fs;

    fn create_test_steps() -> Vec<Step> {
        vec![
            Step {
                title: "Install Dependencies".to_string(),
                description: "Run npm install to install all dependencies.".to_string(),
                code_blocks: vec![CodeBlock {
                    language: Some("bash".to_string()),
                    content: "npm install".to_string(),
                    placeholders: vec![],
                }],
                status: StepStatus::Success,
                output: "added 1234 packages".to_string(),
                condition: None,
            },
            Step {
                title: "Configure Environment".to_string(),
                description: "Set up environment variables.".to_string(),
                code_blocks: vec![CodeBlock {
                    language: Some("bash".to_string()),
                    content: "export API_KEY=<API_KEY>".to_string(),
                    placeholders: vec!["API_KEY".to_string()],
                }],
                status: StepStatus::Pending,
                output: String::new(),
                condition: None,
            },
        ]
    }

    #[test]
    fn test_generate_report_summary() {
        let steps = create_test_steps();
        let report = Exporter::generate_report(
            &steps,
            Path::new("README.md"),
            Path::new("/project"),
            &HashMap::new(),
            &HashMap::new(),
            "1.0.0",
        );

        assert_eq!(report.summary.total_steps, 2);
        assert_eq!(report.summary.completed_steps, 1);
        assert_eq!(report.summary.pending_steps, 1);
        assert!((report.summary.completion_percentage - 50.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_export_markdown_template() -> Result<()> {
        let steps = create_test_steps();
        let report = Exporter::generate_report(
            &steps,
            Path::new("README.md"),
            Path::new("/project"),
            &HashMap::new(),
            &HashMap::new(),
            "1.0.0",
        );

        let temp_dir = std::env::temp_dir();
        let output_path = temp_dir.join("test_report.md");

        Exporter::export_markdown(&report, &output_path)?;

        let content = fs::read_to_string(&output_path)?;

        // Check for key elements in the rendered template
        assert!(content.contains("# 🧭 Compass Session Report"));
        assert!(content.contains("**Compass Version:** 1.0.0"));
        assert!(content.contains("| Total Steps | 2 |"));
        assert!(content.contains("Completed | 1"));
        assert!(content.contains("Install Dependencies"));
        assert!(content.contains("npm install"));

        // Cleanup
        let _ = fs::remove_file(output_path);

        Ok(())
    }

    #[test]
    fn test_status_to_string() {
        assert!(Exporter::status_to_string(StepStatus::Success).contains("Success"));
        assert!(Exporter::status_to_string(StepStatus::Failed).contains("Failed"));
        assert!(Exporter::status_to_string(StepStatus::Pending).contains("Pending"));
        assert!(Exporter::status_to_string(StepStatus::Running).contains("Running"));
    }
}
