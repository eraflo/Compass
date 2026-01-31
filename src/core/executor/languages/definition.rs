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
use std::path::{Path, PathBuf};

/// Defines the behavior for handling a specific programming language.
pub trait LanguageDefinition {
    /// Returns the name of the command required to run this language.
    /// Used for dependency checking.
    fn get_required_command(&self) -> &str;

    /// Prepares the code for execution.
    /// This typically involves writing the code to a temporary file.
    /// For compiled languages, this would also involve compilation.
    ///
    /// # Arguments
    ///
    /// * `code` - The source code to prepare.
    /// * `temp_dir` - The directory where temporary files should be created.
    ///
    /// # Returns
    ///
    /// A `PathBuf` to the executable file or script.
    fn prepare(&self, code: &str, temp_dir: &Path) -> Result<PathBuf>;

    /// Gets the command line arguments to run the prepared file.
    ///
    /// # Arguments
    ///
    /// * `prepared_path` - The path returned by `prepare`.
    fn get_run_command(&self, prepared_path: &Path) -> Vec<String>;

    /// Returns a list of dangerous patterns (strings) that should trigger a safety alert.
    /// Examples: "rm -rf", "os.system", etc.
    fn get_dangerous_patterns(&self) -> &[&'static str] {
        &[]
    }

    /// Returns the typical file extension for this language (e.g., "py", "rs").
    #[allow(dead_code)]
    fn get_extension(&self) -> &str;
}
