---
name: debug-helper
description: Provides solutions when a shell command or a Compass step fails.
input_schema:
  type: object
  properties:
    error_message: { type: string }
    context: { type: string, description: "The command that was being run." }
---

# Debug Helper

## Instructions
- When a `compass-manager` action fails, pass the error output to this skill.
- Analyze common Rust/Shell errors (e.g., 'permission denied', 'linker errors').
- Propose a specific fix command to the user.

## Examples
**Input:** "Error: permission denied for ./target/debug/compass"
**Output:** "It seems the binary lacks execution permissions. Try running `chmod +x ./target/debug/compass`."