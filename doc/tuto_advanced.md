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

## Summary of Flags

| Flag | Description |
|------|-------------|
| `-s`, `--sandbox` | Run in Docker container |
| `--image <IMG>` | Docker image to use (default: ubuntu:latest) |
| `--share` | Start a secure Host session (prints unique join URL) |
| `check` | Analyze dependencies without running UI |
| `join <URL>` | Join a remote session as a guest |
| `parse` | Debug output of the parsed tree |
