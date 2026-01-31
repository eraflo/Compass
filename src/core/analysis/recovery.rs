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

use regex::Regex;

#[derive(Debug, Clone)]
pub struct RecoveryRecommendation {
    pub message: String,
    pub fix_command: Option<String>,
}

/// Analyzes stderr output to suggest recovery actions.
pub fn analyze_error(stderr: &str) -> Option<RecoveryRecommendation> {
    // 1. Port already in use
    // Matches: "Address already in use", "EADDRINUSE", "bind: address already in use"
    let re_port =
        Regex::new(r"(?i)(address already in use|EADDRINUSE|bind: address already in use)")
            .unwrap();
    if re_port.is_match(stderr) {
        return Some(RecoveryRecommendation {
            message: "Port seems to be occupied. You might want to kill the process utilizing it."
                .to_string(),
            fix_command: None, // Too risky to auto-kill without knowing the port accurately
        });
    }

    // 2. Permission denied
    if stderr.contains("Permission denied") || stderr.contains("EACCES") {
        return Some(RecoveryRecommendation {
            message: "Permission denied. You might need 'sudo' or check file permissions."
                .to_string(),
            fix_command: None,
        });
    }

    // 3. Module not found (Python)
    // Matches: "ModuleNotFoundError: No module named 'xyz'"
    let re_py_mod = Regex::new(r"ModuleNotFoundError: No module named '([^']+)'").unwrap();
    if let Some(caps) = re_py_mod.captures(stderr) {
        let module = caps.get(1).map_or("", |m| m.as_str());
        return Some(RecoveryRecommendation {
            message: format!("Python module '{}' is missing.", module),
            fix_command: Some(format!("pip install {}", module)),
        });
    }

    // 4. Command not found
    // Matches: "command not found", "is not recognized as an internal or external command"
    if stderr.contains("command not found") || stderr.contains("not recognized as an internal") {
        return Some(RecoveryRecommendation {
            message: "Command not found. Ensure it is installed and in your PATH.".to_string(),
            fix_command: None,
        });
    }

    // 5. Apt lock (Linux)
    if stderr.contains("Could not get lock /var/lib/dpkg/lock") {
        return Some(RecoveryRecommendation {
            message: "APT database is locked. Another process might be installing software."
                .to_string(),
            fix_command: Some("sudo fuser -v /var/lib/dpkg/lock".to_string()),
        });
    }

    None
}
