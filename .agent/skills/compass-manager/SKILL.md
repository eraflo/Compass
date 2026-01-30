---
name: compass-manager
description: Pilot the Compass Rust binary to parse README structures or check system dependencies.
input_schema:
  type: object
  properties:
    action:
      type: string
      enum: [parse, check]
      description: "The action to perform: 'parse' for structure analysis, 'check' for dependency validation."
    file_path:
      type: string
      description: "Path to the Markdown file (e.g., 'README.md')."
  required: [action, file_path]
---

# Compass Manager

## Instructions
- Use this tool when a user asks to "analyze", "setup", or "onboard" a project.
- Always run `check` before suggesting any command execution to ensure the environment is safe.
- If the binary is not yet compiled, advise the user to run `cargo build` first.

## Pre-conditions (Grounding)
- Before calling this skill, verify that the `file_path` exists using the filesystem tool.
- Verify that `Cargo.toml` is present in the root directory.

## Error Handling Instructions
- If the script returns "Compass Error: command not found", suggest the user to run `cargo build`.
- If the output contains "Parser logic coming soon", inform the user that this module is still under development.

## Examples
**User:** "Show me the steps in this README."
**Agent:** Calls `run_compass.py` with `action="parse", file_path="README.md"`.

**User:** "Can I run this project on my machine?"
**Agent:** Calls `run_compass.py` with `action="check", file_path="README.md"`.