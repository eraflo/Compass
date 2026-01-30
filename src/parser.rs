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

use crate::models::{CodeBlock, Step};
use pulldown_cmark::{Event, Parser, Tag};

/// Parses a Markdown string into a sequence of steps.
pub fn parse_readme(content: &str) -> Vec<Step> {
    let parser = Parser::new(content);
    let mut steps = Vec::new();
    let mut current_step: Option<Step> = None;
    let mut in_heading = false;
    let mut in_code_block = false;
    let mut current_code_lang = None;

    for event in parser {
        match event {
            Event::Start(Tag::Heading { .. }) => {
                // If we were already in a step, push it to the list
                if let Some(step) = current_step.take() {
                    steps.push(step);
                }
                current_step = Some(Step::default());
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
                        } else {
                            step.code_blocks.push(CodeBlock {
                                language: current_code_lang.clone(),
                                content: text.to_string(),
                            });
                        }
                    } else {
                        step.description.push_str(&text);
                    }
                }
            }
            Event::SoftBreak | Event::HardBreak | Event::End(pulldown_cmark::TagEnd::Paragraph) => {
                // Add a new line to the description
                if let Some(ref mut step) = current_step {
                    if !in_heading && !in_code_block {
                        step.description.push('\n');
                    }
                }
            }
            _ => {}
        }
    }

    // Push the last step if it exists
    if let Some(step) = current_step {
        steps.push(step);
    }

    steps
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple() {
        let content =
            "# Header 1\nDescription 1\n```rust\nfn main() {}\n```\n# Header 2\nDescription 2";
        let steps = parse_readme(content);

        assert_eq!(steps.len(), 2);
        assert_eq!(steps[0].title, "Header 1");
        assert_eq!(steps[0].description.trim(), "Description 1");
        assert_eq!(steps[0].code_blocks.len(), 1);
        assert_eq!(steps[0].code_blocks[0].language.as_deref(), Some("rust"));
        assert_eq!(steps[0].code_blocks[0].content.trim(), "fn main() {}");

        assert_eq!(steps[1].title, "Header 2");
        assert_eq!(steps[1].description.trim(), "Description 2");
    }
}
