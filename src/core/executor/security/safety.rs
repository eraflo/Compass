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

/// Detects dangerous command patterns that require user confirmation.
pub struct SafetyShield;

impl SafetyShield {
    /// Checks if a command content contains any blacklisted patterns.
    ///
    /// Returns `Some(pattern)` if a dangerous pattern is found, `None` otherwise.
    pub fn check(cmd_content: &str, patterns: &[&'static str]) -> Option<&'static str> {
        patterns
            .iter()
            .find(|&&pattern| cmd_content.contains(pattern))
            .copied()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_safety_safe() {
        let patterns = &["rm -rf"];
        assert!(SafetyShield::check("ls -la", patterns).is_none());
    }

    #[test]
    fn test_safety_dangerous() {
        let patterns = &["rm -rf", "mkfs"];
        assert!(SafetyShield::check("rm -rf /", patterns).is_some());
        assert!(SafetyShield::check("sudo mkfs.ext4 /dev/sda1", patterns).is_some());
    }
}
