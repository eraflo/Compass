---
name: markdown-test-gen
description: Generates test Markdown files for the Compass parsing engine.
input_schema:
  type: object
  properties:
    case_type:
      type: string
      enum: [nested, huge, malformed]
      description: "The type of edge case to generate."
  required: [case_type]
---

# Markdown Test Generator

## Instructions
- Use this when the developer mentions "testing the parser" or "adding a new test case".
- After generating the file, inform the user of its location in `tests/fixtures/`.
- Suggest running `cargo test` immediately after generation.

## Examples
**User:** "I'm worried about the parser's speed."
**Agent:** Calls `gen_markdown.py` with `case_type="huge"`.