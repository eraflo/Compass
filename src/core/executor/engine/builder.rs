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

use crate::core::models::Step;
use std::collections::HashMap;

pub struct CommandBuilder;

impl CommandBuilder {
    /// Extracts unique keys of all placeholders required by the step.
    pub fn get_required_placeholders(step: &Step) -> Vec<String> {
        let mut placeholders = Vec::new();
        for block in &step.code_blocks {
            for p in &block.placeholders {
                if !placeholders.contains(p) {
                    placeholders.push(p.clone());
                }
            }
        }
        placeholders
    }

    /// Builds the final command string by substituting variables.
    pub fn build_command(step: &Step, variables: &HashMap<String, String>) -> String {
        let mut content = String::new();
        for block in &step.code_blocks {
            let mut block_content = block.content.clone();
            for (key, val) in variables {
                let target_angle = format!("<{key}>");
                let target_brace = format!("{{{{{key}}}}}");
                block_content = block_content.replace(&target_angle, val);
                block_content = block_content.replace(&target_brace, val);
            }
            content.push_str(&block_content);
            content.push_str("\n");
        }
        content
    }
}
