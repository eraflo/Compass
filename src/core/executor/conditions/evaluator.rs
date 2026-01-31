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

use crate::core::models::Condition;
use std::env;
use std::path::Path;

/// A trait for evaluating execution conditions.
pub trait ConditionEvaluator {
    /// Determines if a condition is met.
    fn evaluate(&self, condition: &Condition) -> bool;
}

/// The standard evaluator implementation using system calls.
#[derive(Default)]
pub struct StandardEvaluator;

impl StandardEvaluator {
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl ConditionEvaluator for StandardEvaluator {
    fn evaluate(&self, condition: &Condition) -> bool {
        match condition {
            Condition::Os(os_name) => {
                let current_os = std::env::consts::OS;
                // Loose matching: "windows" == "windows", "macos" == "macos"
                current_os.eq_ignore_ascii_case(os_name)
            }
            Condition::EnvVarExists(var_name) => env::var(var_name).is_ok(),
            Condition::FileExists(path_str) => Path::new(path_str).exists(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_os_condition() {
        let evaluator = StandardEvaluator::new();
        let current = std::env::consts::OS;

        assert!(evaluator.evaluate(&Condition::Os(current.to_string())));
        assert!(!evaluator.evaluate(&Condition::Os("non_existent_os".to_string())));
    }
}
