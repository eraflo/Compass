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

use serde::Serialize;
use std::collections::HashMap;

/// Represents a step in the exported report.
#[derive(Debug, Clone, Serialize)]
pub struct ExportedStep {
    /// The step number (1-indexed).
    pub number: usize,
    /// The title of the step.
    pub title: String,
    /// The description of the step.
    pub description: String,
    /// The status of the step as a string.
    pub status: String,
    /// The code blocks in this step.
    pub code_blocks: Vec<ExportedCodeBlock>,
    /// The captured output from execution.
    pub output: String,
}

/// Represents a code block in the exported report.
#[derive(Debug, Clone, Serialize)]
pub struct ExportedCodeBlock {
    /// The language of the code block.
    pub language: Option<String>,
    /// The content of the code block.
    pub content: String,
}

/// The complete export report structure.
#[derive(Debug, Clone, Serialize)]
pub struct ExportReport {
    /// Report metadata.
    pub metadata: ReportMetadata,
    /// Summary of the session.
    pub summary: ReportSummary,
    /// All steps with their execution details.
    pub steps: Vec<ExportedStep>,
    /// Environment information.
    pub environment: EnvironmentInfo,
}

/// Metadata about the report itself.
#[derive(Debug, Clone, Serialize)]
pub struct ReportMetadata {
    /// The Compass version used.
    pub compass_version: String,
    /// When the report was generated (ISO 8601).
    pub generated_at: String,
    /// The local timestamp for display.
    pub generated_at_local: String,
    /// The path to the README being processed.
    pub readme_path: String,
}

/// Summary statistics of the session.
#[derive(Debug, Clone, Serialize)]
pub struct ReportSummary {
    /// Total number of steps.
    pub total_steps: usize,
    /// Number of completed (successful) steps.
    pub completed_steps: usize,
    /// Number of failed steps.
    pub failed_steps: usize,
    /// Number of pending steps.
    pub pending_steps: usize,
    /// Number of running steps.
    pub running_steps: usize,
    /// Completion percentage.
    pub completion_percentage: f32,
}

/// Environment information captured at export time.
#[derive(Debug, Clone, Serialize)]
pub struct EnvironmentInfo {
    /// The current working directory.
    pub current_dir: String,
    /// Environment variables set during the session.
    pub env_vars: HashMap<String, String>,
    /// Placeholder values used.
    pub placeholders: HashMap<String, String>,
}
