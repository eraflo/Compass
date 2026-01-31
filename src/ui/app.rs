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

use crate::core::executor::ExecutionManager;
use crate::core::models::Step;
use crate::ui::state::Mode;
use crate::ui::state::modal::ModalState;

use ratatui::widgets::ListState;

/// The main application state for the TUI.
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
    /// Modal interface state
    pub modal: ModalState,
    /// The dangerous pattern detected (for safety alert).
    pub safety_pattern: Option<String>,
    /// Scroll offset for the details panel.
    pub details_scroll: u16,
    /// Total height of the details content (wrapped).
    pub content_height: u16,
    /// Height of the details viewport.
    pub viewport_height: u16,
}

impl App {
    #[must_use]
    pub fn new(steps: Vec<Step>) -> Self {
        let mut list_state = ListState::default();
        if !steps.is_empty() {
            list_state.select(Some(0));
        }
        Self {
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

    pub const fn scroll_details_up(&mut self) {
        self.details_scroll = self.details_scroll.saturating_sub(5);
    }

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

    /// Cancels any active modal.
    pub fn cancel_modal(&mut self) {
        self.mode = Mode::Normal;
        self.modal.input_buffer.clear();
        self.modal.required_placeholders.clear();
        self.safety_pattern = None;
    }
}
