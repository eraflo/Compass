---
name: tui-prototyper
description: Provides Rust code snippets for Ratatui widgets.
input_schema:
  type: object
  properties:
    widget_type:
      type: string
      enum: [layout, block, list]
      description: "The specific TUI component needed."
  required: [widget_type]
---

# TUI Prototyper

## Instructions
- When providing a snippet, explain how it fits into the `src/tui.rs` module.
- Always wrap the returned code in a Rust markdown block for the user.

## Examples
**User:** "How do I split the screen in two?"
**Agent:** Calls `get_snippets.py` with `widget_type="layout"`.