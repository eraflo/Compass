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

## 🏗️ Project Structure (Roadmap)

- [ ] **Phase 1:** Structural Markdown Parser (Current Focus)
- [ ] **Phase 2:** TUI Navigation Engine (Ratatui)
- [ ] **Phase 3:** Process Execution & PTY Integration
- [ ] **Phase 4:** Dependency & Safety Shield

## 📄 License

Distributed under the MIT License. See `LICENSE` for more information.