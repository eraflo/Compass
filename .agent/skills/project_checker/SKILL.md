---
name: project-checker
description: Automated tool to run fmt, clippy, and tests locally to ensure code quality before pushing.
---

# Project Checker Skill

This skill provides a centralized way to verify the codebase's integrity.

## Commands

### Full Check
Runs everything (fmt, clippy, tests).
```powershell
.agent/skills/project_checker/scripts/check.ps1
```

### Individual Steps
- **Format**: `cargo fmt --all`
- **Lint**: `cargo clippy --all-targets --all-features -- -D warnings -D clippy::pedantic -D clippy::nursery`
- **Test**: `cargo test`

## Guideline
Always fix all clippy warnings before merging. Use `let_chains`-style restructuring for nested `if` statements.
