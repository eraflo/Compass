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
        Some("javascript" | "js" | "node") => Box::new(strategies::javascript::JsHandler),
        Some("csharp" | "cs" | "c#") => Box::new(strategies::csharp::CSharpHandler),
        Some("typescript" | "ts") => Box::new(strategies::typescript::TsHandler),
        Some("go" | "golang") => Box::new(strategies::go::GoHandler),
        Some("rust" | "rs") => Box::new(strategies::rust::RustHandler),
        Some("php") => Box::new(strategies::php::PhpHandler),
        Some("ruby" | "rb") => Box::new(strategies::ruby::RubyHandler),
        Some("bash" | "sh" | "zsh") => Box::new(strategies::shell::ShellHandler::new("bash")),
        Some("cmd" | "batch") => Box::new(strategies::shell::ShellHandler::new("cmd")),
        Some("powershell" | "pwsh") => Box::new(strategies::shell::ShellHandler::new("powershell")),
        _ => Box::new(strategies::shell::ShellHandler::new("default")),
    }
}
