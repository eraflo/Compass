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

use anyhow::Result;
use std::fs;
use std::path::{Path, PathBuf};

const MAX_DEPTH: usize = 5;

/// Scans the directory for Compass runbooks (README.md or *.runbook.md).
/// Uses an iterative approach to prevent stack overflow and respects max depth.
pub fn scan_directory(root: &Path) -> Result<Vec<PathBuf>> {
    let mut results = Vec::new();
    let mut stack = vec![(root.to_path_buf(), 0)];

    while let Some((dir, depth)) = stack.pop() {
        if depth > MAX_DEPTH {
            continue;
        }

        let entries = match fs::read_dir(&dir) {
            Ok(e) => e,
            Err(_) => continue, // Skip unreadable directories
        };

        for entry in entries.flatten() {
            let path = entry.path();
            let file_name = match path.file_name().and_then(|n| n.to_str()) {
                Some(n) => n,
                None => continue,
            };

            // Skip hidden files/directories (e.g. .git, .vscode)
            if file_name.starts_with('.') || file_name == "target" || file_name == "node_modules" {
                continue;
            }

            if path.is_dir() {
                stack.push((path, depth + 1));
            } else if is_runbook(&path) {
                results.push(path);
            }
        }
    }

    Ok(results)
}

fn is_runbook(path: &Path) -> bool {
    let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
    // Check for README.md (case-insensitive) or .runbook.md extension
    name.eq_ignore_ascii_case("README.md") || name.ends_with(".runbook.md")
}
