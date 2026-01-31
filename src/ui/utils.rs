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

/// Appends output to the buffer, handling ANSI sequences and line endings.
pub fn append_output(buffer: &mut String, new_data: &str) {
    let cleaned_ansi = clean_ansi(new_data);
    // Normalize line endings and strip raw \r
    let normalized = cleaned_ansi.replace("\r\n", "\n").replace('\r', "");

    // Filter for printable characters to avoid corrupting the TUI view
    for c in normalized.chars() {
        if !c.is_ascii_control() || c == '\n' || c == '\t' {
            buffer.push(c);
        }
    }
}

/// Robust ANSI sequence cleaning.
pub fn clean_ansi(s: &str) -> String {
    // More comprehensive regex for ANSI sequences (CSI, OSC, etc.)
    // We accept any letter [a-zA-Z] as a CSI terminator to handle h/l/n etc.
    let re = Regex::new(
        r"(?x)
        \x1b \[ [0-9;?]* [a-zA-Z]      | # CSI sequences
        \x1b \] .*? (\x07|\x1b\\)      | # OSC sequences
        \x1b [()\#] [0-9a-zA-Z]        | # Escaped shortcuts (G0/G1 sets etc)
        \x1b [A-Z>=\[\]]                 # Simple escape codes
    ",
    )
    .unwrap();

    re.replace_all(s, "").to_string()
}
