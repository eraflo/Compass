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

//! # Configuration Module
//!
//! This module provides persistent configuration management for Compass.
//! It saves user-provided placeholder values per README file, so users don't
//! have to re-enter the same values every time they run the same README.
//!
//! Configuration files are stored in the user's config directory:
//! - Linux: `~/.config/compass/`
//! - macOS: `~/Library/Application Support/compass/`
//! - Windows: `%APPDATA%\compass\`

use anyhow::{Context, Result};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// The application name used for configuration directories.
const APP_NAME: &str = "compass";

/// The organization qualifier (empty for simple app name).
const APP_QUALIFIER: &str = "";

/// The organization name.
const APP_ORGANIZATION: &str = "eraflo";

/// Represents the persistent configuration for a specific README file.
///
/// Each README file gets its own configuration file, identified by a hash
/// of the README's absolute path.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ReadmeConfig {
    /// The original path to the README file (for reference).
    pub readme_path: String,
    /// Stored placeholder values (KEY -> VALUE).
    pub placeholders: HashMap<String, String>,
    /// Last modified timestamp (ISO 8601 format).
    pub last_modified: Option<String>,
}

/// Manages persistent configuration for Compass.
///
/// The `ConfigManager` handles loading and saving user preferences
/// and placeholder values to the filesystem.
#[derive(Debug)]
pub struct ConfigManager {
    /// The base configuration directory.
    config_dir: PathBuf,
    /// Currently loaded configuration (per README).
    current_config: ReadmeConfig,
    /// The config file path for the current README.
    config_file_path: Option<PathBuf>,
}

impl ConfigManager {
    /// Creates a new `ConfigManager` and initializes the config directory.
    ///
    /// # Errors
    ///
    /// Returns an error if the configuration directory cannot be created.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let config_manager = ConfigManager::new()?;
    /// ```
    pub fn new() -> Result<Self> {
        let config_dir = Self::get_config_dir()?;

        // Ensure the config directory exists
        if !config_dir.exists() {
            fs::create_dir_all(&config_dir).with_context(|| {
                format!(
                    "Failed to create config directory: {}",
                    config_dir.display()
                )
            })?;
        }

        Ok(Self {
            config_dir,
            current_config: ReadmeConfig::default(),
            config_file_path: None,
        })
    }

    /// Gets the configuration directory path for Compass.
    ///
    /// Uses the `directories` crate to find the appropriate config location
    /// for the current operating system.
    fn get_config_dir() -> Result<PathBuf> {
        let proj_dirs = ProjectDirs::from(APP_QUALIFIER, APP_ORGANIZATION, APP_NAME)
            .context("Could not determine project directories for configuration")?;

        Ok(proj_dirs.config_dir().to_path_buf())
    }

    /// Generates a unique filename for a README's configuration.
    ///
    /// Uses a simple hash of the canonical path to create a unique identifier.
    fn readme_config_filename(readme_path: &Path) -> String {
        // Use a simple hash based on the path string
        let path_str = readme_path.to_string_lossy();
        let hash: u64 = path_str.bytes().fold(0u64, |acc, b| {
            acc.wrapping_mul(31).wrapping_add(u64::from(b))
        });
        format!("readme_{hash:016x}.json")
    }

    /// Loads the configuration for a specific README file.
    ///
    /// If no configuration exists, returns default values.
    ///
    /// # Arguments
    ///
    /// * `readme_path` - The path to the README file.
    ///
    /// # Errors
    ///
    /// Returns an error if the configuration file exists but cannot be parsed.
    pub fn load_for_readme(&mut self, readme_path: &Path) -> Result<()> {
        let canonical_path = readme_path
            .canonicalize()
            .unwrap_or_else(|_| readme_path.to_path_buf());

        let config_filename = Self::readme_config_filename(&canonical_path);
        let config_file_path = self.config_dir.join(&config_filename);

        self.config_file_path = Some(config_file_path.clone());

        if config_file_path.exists() {
            let content = fs::read_to_string(&config_file_path).with_context(|| {
                format!("Failed to read config file: {}", config_file_path.display())
            })?;

            self.current_config = serde_json::from_str(&content).with_context(|| {
                format!(
                    "Failed to parse config file: {}",
                    config_file_path.display()
                )
            })?;
        } else {
            // Initialize with defaults
            self.current_config = ReadmeConfig {
                readme_path: canonical_path.to_string_lossy().to_string(),
                placeholders: HashMap::new(),
                last_modified: None,
            };
        }

        Ok(())
    }

    /// Saves the current configuration to disk.
    ///
    /// # Errors
    ///
    /// Returns an error if the configuration cannot be written to disk.
    pub fn save(&self) -> Result<()> {
        let config_file_path = self
            .config_file_path
            .as_ref()
            .context("No configuration file path set. Call load_for_readme first.")?;

        // Update timestamp
        let mut config_to_save = self.current_config.clone();
        config_to_save.last_modified = Some(chrono::Utc::now().to_rfc3339());

        let content = serde_json::to_string_pretty(&config_to_save)
            .context("Failed to serialize configuration")?;

        fs::write(config_file_path, content).with_context(|| {
            format!(
                "Failed to write config file: {}",
                config_file_path.display()
            )
        })?;

        Ok(())
    }

    /// Gets a stored placeholder value.
    ///
    /// # Arguments
    ///
    /// * `key` - The placeholder name.
    ///
    /// # Returns
    ///
    /// The stored value if it exists, or `None` otherwise.
    #[must_use]
    #[allow(dead_code)]
    pub fn get_placeholder(&self, key: &str) -> Option<&String> {
        self.current_config.placeholders.get(key)
    }

    /// Sets a placeholder value (in memory).
    ///
    /// Call `save()` to persist the changes to disk.
    ///
    /// # Arguments
    ///
    /// * `key` - The placeholder name.
    /// * `value` - The value to store.
    #[allow(dead_code)]
    pub fn set_placeholder(&mut self, key: String, value: String) {
        self.current_config.placeholders.insert(key, value);
    }

    /// Updates multiple placeholder values at once.
    ///
    /// This is useful for bulk updates from the modal state.
    ///
    /// # Arguments
    ///
    /// * `placeholders` - A map of placeholder names to values.
    pub fn update_placeholders(&mut self, placeholders: &HashMap<String, String>) {
        for (key, value) in placeholders {
            self.current_config
                .placeholders
                .insert(key.clone(), value.clone());
        }
    }

    /// Gets all stored placeholders.
    ///
    /// # Returns
    ///
    /// A reference to the placeholder map.
    #[must_use]
    pub const fn get_all_placeholders(&self) -> &HashMap<String, String> {
        &self.current_config.placeholders
    }

    /// Gets the configuration directory path.
    ///
    /// # Returns
    ///
    /// A reference to the configuration directory path.
    #[must_use]
    #[allow(dead_code)]
    pub fn config_dir(&self) -> &Path {
        &self.config_dir
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_readme_config_filename_uniqueness() {
        let path1 = Path::new("/home/user/project/README.md");
        let path2 = Path::new("/home/user/other/README.md");

        let filename1 = ConfigManager::readme_config_filename(path1);
        let filename2 = ConfigManager::readme_config_filename(path2);

        assert_ne!(filename1, filename2);
    }

    #[test]
    fn test_readme_config_filename_consistency() {
        let path = Path::new("/home/user/project/README.md");

        let filename1 = ConfigManager::readme_config_filename(path);
        let filename2 = ConfigManager::readme_config_filename(path);

        assert_eq!(filename1, filename2);
    }
}
