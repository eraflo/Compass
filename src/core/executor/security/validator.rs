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

use which::which;

/// Validates that the required binaries for a command are present in the system's PATH.
pub struct DependencyValidator;

impl DependencyValidator {
    /// Validates that a specific binary is available in the PATH.
    pub fn validate_binary(binary_name: &str) -> Result<(), String> {
        if which(binary_name).is_err() {
            return Err(format!(
                "Missing dependency: '{}' is not installed or not in PATH.",
                binary_name
            ));
        }
        Ok(())
    }

    /// Validates a command string by checking if its primary binary is available.
    ///
    /// Returns `Ok(())` if the dependency is met, or an error message if missing.
    pub fn validate(cmd_content: &str) -> Result<(), String> {
        let trimmed = cmd_content.trim();
        if trimmed.is_empty() {
            return Ok(());
        }

        // Extract the first word (the potential binary)
        let binary_name = trimmed.split_whitespace().next().unwrap_or("");

        // Basic check: ignore shell builtins like cd, echo, etc. for simple validation
        // (Advanced validation might need more nuance, but this is a good start)
        let builtins = ["cd", "export", "set", "exit", "echo"];
        if builtins.contains(&binary_name) {
            return Ok(());
        }

        match which(binary_name) {
            Ok(_) => Ok(()),
            Err(_) => Err(format!(
                "Requirement not met: '{binary_name}' is not installed."
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validator_builtin() {
        assert!(DependencyValidator::validate("cd /tmp").is_ok());
    }

    #[test]
    fn test_validator_common_binary() {
        // 'cargo' should exist in the environment where tests are run
        assert!(DependencyValidator::validate("cargo build").is_ok());
    }

    #[test]
    fn test_validator_missing_binary() {
        assert!(DependencyValidator::validate("this-binary-certainly-does-not-exist").is_err());
    }
}
