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

use std::collections::HashMap;

/// Manages state for the placeholder input modal.
#[derive(Debug, Default)]
pub struct ModalState {
    /// Buffer for user input in the modal.
    pub input_buffer: String,
    /// Store for variable values (KEY -> VALUE).
    pub variable_store: HashMap<String, String>,
    /// List of placeholders required for the current step.
    pub required_placeholders: Vec<String>,
    /// Index of the currently active placeholder being filled.
    pub current_placeholder_idx: usize,
}

impl ModalState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Resets the modal state for a new interaction.
    pub fn reset(&mut self, required: Vec<String>) {
        self.input_buffer.clear();
        self.required_placeholders = required;
        self.current_placeholder_idx = 0;
    }
}
