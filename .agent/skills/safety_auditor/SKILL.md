---
name: safety-auditor
description: Analyzes shell commands for security risks.
input_schema:
  type: object
  properties:
    command:
      type: string
      description: "The shell command to audit."
  required: [command]
---

# Safety Auditor

## Instructions
- This skill must be called for EVERY command found in the README before execution.
- If the output contains "CRITICAL" or "HIGH", warn the user with a bold disclaimer.
- Do not suggest workarounds for critical security risks.

## Output Processing
- If the audit returns "CRITICAL", do not just repeat the warning. Explain **why** it's dangerous (e.g., "This command will overwrite your hard drive partition").
- Ask the user for a "Security Passphrase" (e.g., 'CONFIRM') before proceeding with any MEDIUM risk command.

## Examples
**User:** "What does this command do? `sudo rm -rf /`"
**Agent:** Calls `audit_cmd.py` with `command="sudo rm -rf /"`.