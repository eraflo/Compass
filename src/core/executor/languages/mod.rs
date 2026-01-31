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

pub mod definition;
pub mod strategies;

use definition::LanguageDefinition;

pub fn get_language_handler(lang_id: Option<&str>) -> Box<dyn LanguageDefinition> {
    match lang_id {
        Some("python" | "py") => Box::new(strategies::python::PythonHandler),
        // Add more languages here
        _ => Box::new(strategies::shell::ShellHandler),
    }
}
