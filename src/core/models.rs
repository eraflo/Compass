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

use serde::{Deserialize, Serialize};

/// The status of a step's execution.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum StepStatus {
    #[default]
    Pending,
    Running,
    Success,
    Failed,
    Skipped,
}

/// A condition that must be met for a step to be applicable.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Condition {
    /// The step applies only to a specific OS (linux, macos, windows).
    Os(String),
    /// The step applies only if a specific environment variable exists.
    EnvVarExists(String),
    /// The step applies only if a specific file exists.
    FileExists(String),
}

/// A block of code extracted from a Markdown file.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CodeBlock {
    /// The language of the code block (e.g., "rust", "bash").
    pub language: Option<String>,
    /// The raw content of the code block.
    pub content: String,
    /// Placeholders found in this block (e.g., "`VARIABLE_NAME`").
    pub placeholders: Vec<String>,
}

/// A parsing step representing a section of the README.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Step {
    /// The title of the step (extracted from a header).
    pub title: String,
    /// The accumulated text description between headers.
    pub description: String,
    /// A list of code blocks found within this section.
    pub code_blocks: Vec<CodeBlock>,
    /// The current execution status of this step.
    pub status: StepStatus,
    /// The captured output (stdout and stderr) from the last execution.
    pub output: String,
    /// An optional condition for this step (e.g., OS-specific).
    pub condition: Option<Condition>,
}

impl Step {
    /// Checks if the step is executable (i.e., has code blocks).
    pub const fn is_executable(&self) -> bool {
        !self.code_blocks.is_empty()
    }
}
