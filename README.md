# 🧭 Compass

**Stop copy-pasting. Start navigating.**

`compass` is a blazingly fast, terminal-based interactive runbook runner built in Rust. It transforms static `README.md` files into executable, step-by-step journeys, ensuring seamless developer onboarding and reliable infrastructure management.

---

## ⚡ Why Compass?

Documentation is often a graveyard of outdated commands and copy-paste errors. Compass bridges the gap between *reading* and *doing*:

- **Zero-Config:** Works out of the box with any standard Markdown file.
- **Interactive TUI:** A beautiful terminal interface powered by `ratatui`.
- **Pre-flight Checks:** Automatically detects missing dependencies before you run a command.
- **Safe by Design:** Edit commands on the fly and review impact before execution.
- **Blazingly Fast:** Single binary, no runtime, built with 🦀 Rust.

## 🚀 Getting Started

### Installation (Coming Soon)

```bash
cargo install compass
```

### Usage

Simply point Compass to any Markdown file:

```bash
compass README.md
```

## 🛠️ How it works

1. **Scan:** Compass parses the Markdown structure, identifying headers as steps and code blocks as executable actions.
2. **Validate:** It checks your local system for required binaries (npm, docker, terraform, etc.).
3. **Navigate:** Use your arrow keys to move through the guide.
4. **Execute:** Press `Enter` to run a step. Compass maintains the environment and directory context between steps.

## 🤝 Real-time Collaboration (Secure)

Compass allows you to work together on a runbook in real-time.

### Host a Session
Pass the `--share` flag to start a secure session:
```bash
compass tui RELEASING.md --share
```
Compass will generate a **Self-Signed Certificate** and a unique **PIN**.
Share the provided secure link (e.g., `wss://192.168.1.15:3030/?pin=...`) with your team.

### Join a Session
Paste the secure link to join as a guest:
```bash
compass join "wss://192.168.1.15:3030/?pin=a1b2c3d4..."
```

> **Security Note:** Connections are End-to-End Encrypted (TLS 1.3). We use **Certificate Pinning** to prevent Man-In-The-Middle attacks without needing a centralized Certificate Authority. Authentication is enforced via the PIN.

## 📄 License

Distributed under the MIT License. See `LICENSE` for more information.