# Compass Navigator for VS Code

<div align="center">
  <img src="media/compass.png" alt="Compass Logo" width="128" />
</div>

**Compass Navigator** brings the power of [Compass](https://github.com/eraflo/compass) directly into Visual Studio Code. Turn any Markdown file into an interactive, executable runbook without leaving your editor.

## ✨ Features

- **Interactive Sidebar**: visualizes steps extracted from your Markdown file.
- **One-Click Execution**: run code blocks directly from the tree view.
- **Real-time Logs**: view standard output and commands as they stream from the Compass backend.
- **Status Tracking**: visual indicators for Success/Failure states of each step.

## 🚀 Getting Started

### Prerequisites

This extension requires the **Compass CLI** to be installed and available in your system PATH.

```bash
cargo install compass-cli
```
*(Or build from source via `cargo install --path .`)*

### Usage

1. Open a **Markdown (.md)** file in VS Code.
2. Open the Command Palette (`Ctrl+Shift+P` / `Cmd+Shift+P`).
3. Run the command: **`Compass: Start Navigator`**.
4. The **Compass Navigator** view will appear in the Explorer sidebar.
5. Click on the "Play" button next to any step to execute it.

## ⚙️ Extension Settings

This extension contributes the following settings:

* `compass.binaryPath`: Specifies the absolute path to the `compass` executable.
    * Default: `compass` (assumes it is in your system PATH).

## 🔧 Installation

### From Marketplace
Search for **"Compass Navigator"** in the VS Code Extensions view and click Install.

### From Source (For Developers)

1. Clone the repository.
2. Navigate to `vscode-extension/`.
3. Install dependencies: `npm install`.
4. Compile: `npm run compile`.
5. Press `F5` to launch a new Extension Development Host window.

## 📄 License

Apache 2.0