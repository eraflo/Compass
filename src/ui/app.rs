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

use crate::core::config::ConfigManager;
use crate::core::executor::ExecutionManager;
use crate::core::models::Step;
use crate::ui::state::Mode;
use crate::ui::state::modal::ModalState;

use ratatui::widgets::ListState;
use std::path::PathBuf;

/// The current version of Compass (synchronized with Cargo.toml).
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// The main application state for the TUI.
///
/// This struct holds all the state necessary to render the UI and handle
/// user interactions. It manages steps, execution, configuration, and modals.
pub struct App {
    /// The list of steps parsed from the README.
    pub steps: Vec<Step>,
    /// The current state of the step list selection.
    pub list_state: ListState,
    /// Whether the application should exit.
    pub should_quit: bool,
    /// The execution manager.
    pub execution_manager: ExecutionManager,
    /// Current UI mode.
    pub mode: Mode,
    /// Modal interface state.
    pub modal: ModalState,
    /// The dangerous pattern detected (for safety alert).
    pub safety_pattern: Option<String>,
    /// Scroll offset for the details panel.
    pub details_scroll: u16,
    /// Total height of the details content (wrapped).
    pub content_height: u16,
    /// Height of the details viewport.
    pub viewport_height: u16,
    /// Path to the README file being processed.
    pub readme_path: PathBuf,
    /// Configuration manager for persistent settings.
    pub config_manager: Option<ConfigManager>,
    /// Export notification message (success/error).
    pub export_message: Option<(bool, String)>,
    /// Scroll offset for the help modal.
    pub help_scroll: u16,
    /// Indicates if the README is loaded from a remote source.
    pub is_remote: bool,
}

impl App {
    /// Creates a new `App` with the given steps and README path.
    ///
    /// # Arguments
    ///
    /// * `steps` - The list of steps parsed from the README.
    /// * `readme_path` - The path to the README file.
    /// * `is_remote` - Whether the file is remote.
    ///
    /// # Returns
    ///
    /// A new `App` instance ready for rendering.
    #[must_use]
    pub fn new(steps: Vec<Step>, readme_path: PathBuf, is_remote: bool) -> Self {
        let mut list_state = ListState::default();
        if !steps.is_empty() {
            list_state.select(Some(0));
        }

        // Initialize configuration manager
        let config_manager = ConfigManager::new().ok();

        let app = Self {
            steps,
            list_state,
            should_quit: false,
            execution_manager: ExecutionManager::new(),
            mode: Mode::Normal,
            modal: ModalState::new(),
            safety_pattern: None,
            details_scroll: 0,
            content_height: 0,
            viewport_height: 0,
            readme_path,
            config_manager,
            export_message: None,
            help_scroll: 0,
            is_remote,
        };

        app
    }

    /// Loads configuration for the current README and pre-fills placeholders.
    ///
    /// This should be called after creating the App to restore any
    /// previously saved placeholder values.
    pub fn load_config(&mut self) {
        #[allow(clippy::collapsible_if)]
        if let Some(ref mut config) = self.config_manager {
            if config.load_for_readme(&self.readme_path).is_ok() {
                // Pre-fill the modal's variable store with saved values
                for (key, value) in config.get_all_placeholders() {
                    self.modal.variable_store.insert(key.clone(), value.clone());
                }
            }
        }
    }

    /// Saves the current placeholder values to the configuration.
    ///
    /// This persists the user's input so it can be restored on next launch.
    pub fn save_config(&mut self) {
        if let Some(ref mut config) = self.config_manager {
            config.update_placeholders(&self.modal.variable_store);
            let _ = config.save(); // Ignore errors silently for now
        }
    }

    /// Selects the next step in the list.
    pub fn next(&mut self) {
        if self.mode != Mode::Normal {
            return;
        }
        let i = match self.list_state.selected() {
            Some(i) => {
                if i >= self.steps.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
        self.details_scroll = 0;
    }

    /// Selects the previous step in the list.
    pub fn previous(&mut self) {
        if self.mode != Mode::Normal {
            return;
        }
        let i = match self.list_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.steps.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
        self.details_scroll = 0;
    }

    /// Scrolls the details panel up.
    pub const fn scroll_details_up(&mut self) {
        self.details_scroll = self.details_scroll.saturating_sub(5);
    }

    /// Scrolls the details panel down.
    pub const fn scroll_details_down(&mut self) {
        let max_scroll = self.content_height.saturating_sub(self.viewport_height);
        if self.details_scroll < max_scroll {
            // min is const stable for u16 since 1.32
            self.details_scroll = if self.details_scroll.saturating_add(5) < max_scroll {
                self.details_scroll.saturating_add(5)
            } else {
                max_scroll
            };
        }
    }

    /// Scrolls the help modal up.
    pub const fn scroll_help_up(&mut self) {
        self.help_scroll = self.help_scroll.saturating_sub(1);
    }

    /// Scrolls the help modal down.
    pub const fn scroll_help_down(&mut self) {
        self.help_scroll = self.help_scroll.saturating_add(1);
    }

    /// Cancels any active modal.
    pub fn cancel_modal(&mut self) {
        self.mode = Mode::Normal;
        self.modal.input_buffer.clear();
        self.modal.required_placeholders.clear();
        self.safety_pattern = None;
        self.export_message = None;
    }

    /// Gets the count of completed steps.
    #[must_use]
    pub fn completed_count(&self) -> usize {
        use crate::core::models::StepStatus;
        self.steps
            .iter()
            .filter(|s| s.is_executable() && s.status == StepStatus::Success)
            .count()
    }

    /// Gets the count of failed steps.
    #[must_use]
    pub fn failed_count(&self) -> usize {
        use crate::core::models::StepStatus;
        self.steps
            .iter()
            .filter(|s| s.is_executable() && s.status == StepStatus::Failed)
            .count()
    }

    /// Gets the count of total executable steps.
    #[must_use]
    pub fn total_executable_steps(&self) -> usize {
        self.steps.iter().filter(|s| s.is_executable()).count()
    }
}
