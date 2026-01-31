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

pub mod modal;

use crate::core::models::StepStatus;
use std::collections::HashMap;
use std::path::PathBuf;

/// Messages sent from execution threads to the main UI loop.
pub enum ExecutionMessage {
    /// Partial output from a PTY.
    OutputPartial(usize, String),
    /// Execution finished with status and final context.
    Finished(usize, StepStatus, PathBuf, HashMap<String, String>),
}

/// The various states the application UI can be in.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Mode {
    Normal,
    /// Waiting for user input to fill placeholders.
    InputModal,
    /// Waiting for confirmation of a dangerous command.
    SafetyAlert,
}
