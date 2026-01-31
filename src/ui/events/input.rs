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

use crate::ui::app::App;
use crate::ui::events::handlers;
use crate::ui::state::Mode;
use crossterm::event::{KeyCode, KeyEvent};

/// Handles key events and dispatches actions to the App.
pub fn handle_input(app: &mut App, key: KeyEvent) {
    match app.mode {
        Mode::Normal => match key.code {
            KeyCode::Char('q') => app.should_quit = true,
            KeyCode::Down | KeyCode::Char('j') => app.next(),
            KeyCode::Up | KeyCode::Char('k') => app.previous(),
            KeyCode::Char('J') => app.scroll_details_down(),
            KeyCode::Char('K') => app.scroll_details_up(),
            KeyCode::PageDown => app.scroll_details_down(),
            KeyCode::PageUp => app.scroll_details_up(),
            KeyCode::Enter => {
                handlers::execute_selected(app);
            }
            _ => {}
        },
        Mode::InputModal => match key.code {
            KeyCode::Enter => {
                handlers::submit_input(app);
            }
            KeyCode::Esc => {
                app.cancel_modal();
            }
            KeyCode::Char(c) => {
                app.modal.input_buffer.push(c);
            }
            KeyCode::Backspace => {
                app.modal.input_buffer.pop();
            }
            _ => {}
        },
        Mode::SafetyAlert => match key.code {
            KeyCode::Enter => {
                handlers::confirm_safety(app);
            }
            KeyCode::Esc => {
                app.cancel_modal();
            }
            _ => {}
        },
    }
}
