# Compass Advanced Tutorial

This guide covers advanced features like Sandboxing, Dependency Checking, and Remote Execution.

## 1. Safety First: The Sandbox Mode

For untrusted scripts or complex setups where you don't want to pollute your host system, use the Sandbox.

### Requirements
- Docker must be installed.

### How to use
Add the `--sandbox` (or `-s`) flag:

```bash
compass tui --sandbox examples/safety_test.md
```

By default, this uses `ubuntu:latest`. You can specify a custom image:

```bash
compass tui -s --image python:3.9-slim examples/dependency_test.md
```

**What happens?**
Compass spins up a Docker container, mounts your current workspace to `/workspace`, and executes commands inside that container.

> Note: Compass will attempt to auto-start Docker on Windows and macOS if it's not running.

## 2. Managing Dependencies

Before starting a complex tutorial, you might want to know if you have the necessary tools installed.

### The Check Command
Run the `check` subcommand against a markdown file:

```bash
compass check examples/dependency_test.md
```

Output:
```text
✅ Present:
   - cargo
   - git

❌ Missing:
   - python
```

This statically analyzes the code blocks for common commands (like `cargo`, `npm`, `python`) and checks your PATH.

## 3. Remote Execution

You can run a README directly from GitHub without cloning the repo first.

```bash
compass tui https://github.com/eraflo/compass/blob/main/README.md
```

Compass handles downloading the content and rewriting relative links (like images) so they point to the correct remote URL.

## 4. Dangerous Commands

Try running `examples/safety_test.md`. It contains a simulated dangerous command:

```bash
rm -rf /
```

When you try to execute this step with `x`, Compass detects the pattern and interrupts with a **Safety Warning**. You must explicitly confirm the action.

## Summary of Flags

| Flag | Description |
|------|-------------|
| `-s`, `--sandbox` | Run in Docker container |
| `--image <IMG>` | Docker image to use (default: ubuntu:latest) |
| `check` | Analyze dependencies without running UI |
| `parse` | Debug output of the parsed tree |
