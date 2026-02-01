# 🧭 Compass

<p align="center">
  <img src="doc/logo.png" alt="Compass Logo" width="150" height="auto" />
</p>

**Stop copy-pasting. Start navigating.**

`compass` is a blazingly fast, terminal-based interactive runbook runner built in Rust. It transforms static `README.md` files into executable, step-by-step journeys, ensuring seamless developer onboarding and reliable infrastructure management.

---

## ⚡ Why Compass?

Documentation is often a graveyard of outdated commands and copy-paste errors. Compass bridges the gap between *reading* and *doing*:

- **Zero-Config:** Works out of the box with any standard Markdown file.
- **Interactive TUI:** A beautiful terminal interface powered by `ratatui`.
- **Pre-flight Checks:** Automatically detects missing dependencies before you run a command.
- **Automated Hooks:** Define pre-run, post-run, and failure handlers directly in your frontmatter.
- **Ecosystem Ready:** Discover community runbooks, scan local repos, and integrate via headless mode.
- **VS Code Extension:** Run your runbooks directly within your favorite editor using the integrated Compass Navigator.
- **Secure Collaboration:** Real-time, encrypted multiplayer mode for pair programming on runbooks.
- **Blazingly Fast:** Single binary, no runtime, built with 🦀 Rust.

## 🚀 Getting Started

### Installation

```bash
cargo install compass-cli
```

### Usage

Simply point Compass to any Markdown file:

```bash
compass tui README.md
```

## 🛠️ Key Features

### 1. Interactive TUI
Compass parses the Markdown structure, identifying headers as steps and code blocks as executable actions. Navigate with arrow keys, edit on the fly, and execute.

### 2. Ecosystem & Discovery
Compass isn't just a runner; it's a platform.

- **Deep Scan**: Find all runbooks in your project.
  ```bash
  compass scan ./projects
  ```

- **Compass Hub**: Search for community-maintained runbooks (e.g., Kubernetes setups, React starters).
  ```bash
  compass search "react"
  ```

### 3. Event Hooks (Automation)
Add a YAML frontmatter to your markdown to trigger actions automatically.

```yaml
---
pre_run: "echo 'Initializing environment...'"
on_failure: "echo 'Step failed! Alerting team...'"
---
```

### 4. Headless Mode (IDE Integration)
Want to build a VS Code extension or an automated agent?
```bash
compass tui --headless README.md
```
This starts a JSON-RPC 2.0 server over Stdio, allowing programmatic control of the runner.

## 🤝 Real-time Collaboration (Secure)

Compass allows you to work together on a runbook in real-time.

### Host a Session
Pass the `--share` flag to start a secure session:
```bash
compass tui RELEASING.md --share
```
Compass generates a secure link with a pinned certificate (TLS 1.3). Share this link with your peer.

### Join a Session
Paste the secure link to join as a guest:
```bash
compass join "wss://192.168.1.15:3030/?pin=a1b2c3d4..."
```

> **Security Note:** Connections are End-to-End Encrypted (TLS 1.3). We use **Certificate Pinning** to prevent Man-In-The-Middle attacks without needing a centralized Certificate Authority. Authentication is enforced via the PIN.

## 📄 License

Distributed under the MIT License. See `LICENSE` for more information.