# Compass Capabilities

Compass is an interactive Navigator for Markdown READMEs, designed to make complex setups and tutorials executable and safe.

## Core Features

### 1. Interactive TUI (Text User Interface)
- **Navigation**: Parse headers (H1-H6) as steps. Navigate between them using Arrow keys or `j/k/h/l` (Vim style).
- **Execution**: Execute code blocks found within a section directly in your terminal.
- **State Tracking**: Visually tracks which steps have been visited or executed.

### 2. Intelligent Execution Engine
- **PTY Support**: Allocation of a pseudo-terminal for executed commands, preserving colors and interactivity (e.g., confirmation prompts).
- **Environment Management**: Commands run in the current shell environment, or an isolated one (see Sandbox).
- **Shell Awareness**: Detects shell types (PowerShell/CMD on Windows, Bash/Sh on Linux/Mac).

### 3. Safety & Security
- **Confirmation Prompts**: Hazardous commands (like `rm -rf`) trigger a confirmation popup before execution.
- **Sandbox Mode**: Isolate execution inside a Docker container using the `--sandbox` flag.
    - Mounts the current workspace read-only (or standard rw).

### 4. Integrations
- **Headless Mode**: Run Compass as a JSON-RPC server (`--headless`) to integrate with external tools (IDEs, CI pipelines).
- **VS Code Extension**: A dedicated "Compass Navigator" extension allows developers to execute runbooks directly from the editor sidebar, with real-time log streaming.
    - Maps temporary script directories.
    - Rewrites paths to be container-compatible.
- **Dependency Checks**: The `compass check` command scans a README for required tools (e.g., `cargo`, `npm`, `docker`) and verifies their presence in PATH.

### 4. Smart Fetching
- **Remote Files**: Can launch directly from a URL (e.g., `compass tui https://github.com/user/repo/README.md`).
- **URL Rewriting**: Automatically rewrites relative image links in remote Markdown files to absolute raw.githubusercontent links, so images render correctly in supported terminals or export formats.

### 5. Docker Integration
- **Auto-detection**: Checks if Docker is running before launching sandbox mode.
- **Auto-start**: On Windows and macOS, attempts to launch Docker Desktop if it is not running.
- **Image Selection**: Custom docker images can be specified via `--image`.

### 6. Collaboration (Secure)
- **Real-time Sessions**: Multiple users can work on the same runbook session.
- **Role-Based Access Control**: 
    - **Host (Driver)**: Controls navigation, executes commands, broadcasts state. Full Control.
    - **Guest (Observer)**: View-only access. Follows navigation and execution output in real-time. Cannot execute commands.
- **Security Architecture (Zero-Trust)**:
    - **Encryption**: All traffic runs over TLS 1.3 (WSS).
    - **Certificate Pinning**: The Host generates a self-signed cert on the fly. Detailed fingerprints replace CA validation.
    - **Authentication**: A unique PIN token guards access. This PIN serves as both the certificate validator and the access key.

## Architecture

- **Core**: Parsing, Model definitions, Execution logic.
- **UI**: Ratatui-based interface.
- **Security**: Validator logic for command safety.
