---
name: test-assistant
description: Runs Rust tests, analyzes failures, and suggests new test cases for Compass.
input_schema:
  type: object
  properties:
    action:
      type: string
      enum: [run_all, run_unit, explain_failure]
      description: "The testing action to perform."
    test_name:
      type: string
      description: "Optional: name of a specific test to run or explain."
  required: [action]
---

# Test Assistant

## Instructions
- Use this skill whenever the user says "run tests" or "why is this test failing?".
- **run_all**: Executes all workspace tests.
- **run_unit**: Runs only unit tests in a specific module.
- **explain_failure**: Analyzes the output of a failed test and suggests a fix in the Rust code.

## Grounding
- Before running tests, ensure the agent is in the root directory where `Cargo.toml` is located.
- If `cargo test` fails to compile, call the `debug-helper` skill.

## Examples
**User:** "Run all tests and tell me if everything is green."
**Agent:** Calls `test_runner.py` with `action="run_all"`.