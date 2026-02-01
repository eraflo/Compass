# Compass Advanced Tutorial

<img src="logo.png" alt="Compass Logo" width="100" align="right" />

This guide covers advanced features like Sandboxing, Dependency Checking, Headless Remote Execution.

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

## 3. Automation with Event Hooks

Hooks allow you to trigger actions automatically based on the lifecycle of your runbook execution. This is powerful for setting up environments or reporting status.

### The Frontmatter Configuration

Add a YAML frontmatter block at the very top of your Markdown file:

```markdown
---
pre_run: "echo 'Starting deployment sequence...' && mkdir -p logs"
post_run: "echo 'Cleanup complete.' && rm -rf tmp/"
on_failure: "echo 'CRITICAL FAILURE: Notify admin'"
---

# My Deployment Runbook
```

- **`pre_run`**: Executes BEFORE the runbook opens. Useful for checks or setup.
- **`post_run`**: Executes AFTER you exit the runbook (if successful).
- **`on_failure`**: Executes if a step fails or the runbook crashes.

> **Security Note:** When running a file with hooks, Compass will ask for your confirmation before executing them unless you are in `--headless` mode or pass a trusted flag.

## 4. The Compass Ecosystem

Compass allows you to manage and discover runbooks across your entire system.

### Scanning for Runbooks
Lost track of your `README.md` files? Scan a directory to find all valid Compass runbooks.

```bash
compass scan ./my-projects
```

### The Hub
Search the global community registry for standard runbooks.

```bash
compass search "docker"
```

If you find a runbook you like, you can clone or run it directly (future feature).


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

## 5. Collaboration Mode (Team Work)

Compass allows you to guide a team member through a procedure remotely, ensuring you both see the same steps.

### Hosting (The Guide)
You can share your local session with others securely.

```bash
compass tui RELEASING.md --share
```

When you run this:
1. Compass generates a unique **TLS Certificate** and a **PIN**.
2. It displays a secure link: `wss://<IP>:3030/?pin=<FINGERPRINT>`.
3. Share this link with your teammate.

### Joining (The Follower)
The guest simply runs the join command:

```bash
compass join "wss://192.168.1.50:3030/?pin=a1b2c3d4..."
```

> **Note:** Guests are in **Read-Only** mode. They can follow the navigation and see output in real-time, but for security reasons, **only the Host** can actually execute commands on their machine.

### Security Details
- **Zero-Trust**: We do not rely on public Certificate Authorities.
- **Pinning**: The `pin` parameter contains the SHA256 hash of the server's certificate. The client will **only** connect if the server proves it owns the certificate matching this exact hash.
- **Encryption**: The connection is fully encrypted (TLS 1.3).
- **Authentication**: The server rejects any connection that does not know the PIN.

## 4. Headless Mode & IDE Integration

Compass can run as a **JSON-RPC server**, allowing other tools (like IDEs) to drive the execution.

### Enabling Headless Mode
```bash
compass --headless path/to/README.md
```
In this mode, Compass reads JSON requests from `stdin` and streams logs/results to `stdout`.

### VS Code Integration
This is the backend that powers the **Compass Navigator** extension. It allows you to:
1. Visualize the runbook tree in VS Code.
2. Click "Play" on steps.
3. See logs in the Output panel.
4. Stop execution automatically if a step fails.

## Summary of Flags

| Flag | Description |
|------|-------------|
| `-s`, `--sandbox` | Run in Docker container |
| `--image <IMG>` | Docker image to use (default: ubuntu:latest) |
| `--headless` | Run in JSON-RPC Headless mode for IDE integration |
| `--share` | Start a secure Host session (prints unique join URL) |
| `check` | Analyze dependencies without running UI |
| `join <URL>` | Join a remote session as a guest |
| `parse` | Debug output of the parsed tree |
