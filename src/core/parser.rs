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

use crate::core::ecosystem::hooks::HookConfig;
use crate::core::models::{CodeBlock, Condition, Step};
use pulldown_cmark::{Event, Parser, Tag};
use regex::Regex;

/// Parses a Markdown string into a sequence of steps and optional hook configuration.
pub fn parse_readme(content: &str) -> (Vec<Step>, Option<HookConfig>) {
    let mut current_content = content;
    let mut hook_config = None;

    // Frontmatter parsing
    if let Some(rest) = content.strip_prefix("---")
        && let Some(end_idx) = rest.find("\n---")
    {
        let frontmatter_str = &rest[..end_idx];
        match serde_yaml::from_str::<HookConfig>(frontmatter_str) {
            Ok(config) => {
                hook_config = Some(config);
                // Skip the closing delimiter "\n---" (4 chars)
                if rest.len() > end_idx + 4 {
                    current_content = &rest[end_idx + 4..];
                    // Consume one optional newline if present directly after ---
                    if let Some(s) = current_content.strip_prefix('\n') {
                        current_content = s;
                    } else if let Some(s) = current_content.strip_prefix("\r\n") {
                        current_content = s;
                    }
                } else {
                    current_content = "";
                }
            }
            Err(e) => eprintln!("Failed to parse frontmatter: {}", e),
        }
    }

    let parser = Parser::new(current_content);
    let mut steps = Vec::new();
    let mut current_step: Option<Step> = None;
    let mut in_heading = false;
    let mut in_code_block = false;
    let mut current_code_lang = None;
    let mut active_condition: Option<Condition> = None;

    let re_if = Regex::new(r#"<!--\s*compass:if\s+(\w+)="([^"]+)"\s*-->"#).unwrap();
    let re_endif = Regex::new(r#"<!--\s*compass:endif\s*-->"#).unwrap();

    for event in parser {
        match event {
            Event::Html(cow_str) => {
                let text = cow_str.trim();

                if let Some(caps) = re_if.captures(text) {
                    let key = caps.get(1).map_or("", |m| m.as_str());
                    let val = caps.get(2).map_or("", |m| m.as_str());

                    active_condition = match key {
                        "os" => Some(Condition::Os(val.to_string())),
                        "env_var_exists" => Some(Condition::EnvVarExists(val.to_string())),
                        "file_exists" => Some(Condition::FileExists(val.to_string())),
                        _ => None, // Unknown condition type
                    };
                } else if re_endif.is_match(text) {
                    active_condition = None;
                }
            }
            Event::Start(Tag::Heading { .. }) => {
                // If we were already in a step, push it to the list
                if let Some(step) = current_step.take() {
                    steps.push(step);
                }
                current_step = Some(Step {
                    condition: active_condition.clone(),
                    ..Default::default()
                });
                in_heading = true;
            }
            Event::End(pulldown_cmark::TagEnd::Heading(_)) => {
                in_heading = false;
            }
            Event::Start(Tag::CodeBlock(kind)) => {
                // Currently only support fenced code blocks
                in_code_block = true;
                if let pulldown_cmark::CodeBlockKind::Fenced(lang) = kind {
                    // Detect if language is defined
                    if !lang.is_empty() {
                        current_code_lang = Some(lang.to_string());
                    }
                }
            }
            Event::End(pulldown_cmark::TagEnd::CodeBlock) => {
                in_code_block = false;
                current_code_lang = None;
            }
            Event::Text(text) => {
                // Currently, get content from text
                if let Some(ref mut step) = current_step {
                    if in_heading {
                        step.title.push_str(&text);
                    } else if in_code_block {
                        // If we are in a code block, add the text to the last code block
                        // Else, create a new code block
                        if let Some(last_block) = step.code_blocks.last_mut() {
                            last_block.content.push_str(&text);
                            // Re-extract placeholders if content grows
                            last_block.placeholders = extract_placeholders(&last_block.content);
                        } else {
                            let placeholders = extract_placeholders(&text);
                            step.code_blocks.push(CodeBlock {
                                language: current_code_lang.clone(),
                                content: text.to_string(),
                                placeholders,
                            });
                        }
                    } else {
                        step.description.push_str(&text);
                    }
                }
            }
            Event::SoftBreak | Event::HardBreak | Event::End(pulldown_cmark::TagEnd::Paragraph) => {
                // Add a new line to the description
                if let Some(step) = current_step
                    .as_mut()
                    .filter(|_| !in_heading && !in_code_block)
                {
                    step.description.push('\n');
                }
            }
            _ => {}
        }
    }

    // Push the last step if it exists
    if let Some(step) = current_step {
        steps.push(step);
    }

    (steps, hook_config)
}

/// Extracts placeholders like <VAR> or {{VAR}} from a string.
fn extract_placeholders(text: &str) -> Vec<String> {
    // We restrict placeholders to alphanumeric chars to avoid matching
    // HTML tags, PHP tags (<?php ... ?>), or generics (<T>).
    let re = regex::Regex::new(r"\{{2}([a-zA-Z0-9_-]+)\}{2}|<([a-zA-Z0-9_-]+)>").unwrap();
    let mut placeholders = Vec::new();
    for cap in re.captures_iter(text) {
        if let Some(m) = cap.get(1).or_else(|| cap.get(2)) {
            let name = m.as_str().trim().to_string();
            if !placeholders.contains(&name) {
                placeholders.push(name);
            }
        }
    }
    placeholders
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple() {
        let content =
            "# Header 1\nDescription 1\n```rust\nfn main() {}\n```\n# Header 2\nDescription 2";
        let (steps, _) = parse_readme(content);

        assert_eq!(steps.len(), 2);
        assert_eq!(steps[0].title, "Header 1");
        assert_eq!(steps[0].description.trim(), "Description 1");
        assert_eq!(steps[0].code_blocks.len(), 1);
        assert_eq!(steps[0].code_blocks[0].language.as_deref(), Some("rust"));
        assert_eq!(steps[0].code_blocks[0].content.trim(), "fn main() {}");

        assert_eq!(steps[1].title, "Header 2");
        assert_eq!(steps[1].description.trim(), "Description 2");
    }

    #[test]
    fn test_extract_placeholders() {
        let text = "echo <USER_NAME> and {{API_KEY}}";
        let placeholders = extract_placeholders(text);
        assert_eq!(placeholders.len(), 2);
        assert_eq!(placeholders[0], "USER_NAME");
        assert_eq!(placeholders[1], "API_KEY");
    }

    #[test]
    fn test_parse_with_placeholders() {
        let content = "# Test\n```bash\necho <HELLO>\n```";
        let (steps, _) = parse_readme(content);
        assert_eq!(steps[0].code_blocks[0].placeholders[0], "HELLO");
    }
}
