# Compass Beginner Tutorial

<img src="logo.png" alt="Compass Logo" width="100" align="right" />

Welcome to Compass! This guide will help you get started with interactive README execution.

## Prerequisites
- A working installation of Compass (`cargo install compass-cli`).

## Basic Concepts
Compass treats every Header in a Markdown file as a "Step". Code blocks under that header belong to that step.

## Tutorial: Running the Placeholder Test

We have provided a simple example file to demonstrate navigation and output.

### 1. Launching Compass

**Option A: Terminal (TUI)**
Run the following command in your terminal:

```bash
compass tui examples/placeholder_test.md
```

**Option B: VS Code Extension**
1. Open `examples/placeholder_test.md` in VS Code.
2. Open Command Palette (`Ctrl+Shift+P`) -> `Compass: Start Navigator`.
3. Use the Sidebar to run steps.

### 2. The Interface (TUI)
You will see the **Table of Contents** on the left and the **Details** on the right.

- **Navigate**: Use `Up/Down` arrows or `j/k` to move between steps.
- **Select**: Press `Enter` to focus on a step (or just view details).
- **Toggle View**: Press `Tab` to switch focus between the list and the details pane.

### 3. Executing Code
Navigate to "Step 2: Installation". You will see a code block.

1. Press `x` to execute the code block.
2. A terminal window will appear (simulated) showing the output.
3. Press `Enter` to close the output window and return to the main view.

### 4. Quitting
Press `q` or `Esc` to exit the application.

## Next Steps
Try creating your own `README.md` with:
```markdown
# My Project Setup
## Install
```bash
echo "Installing..."
```
## Run
```bash
echo "Running!"
```
```

Then run `compass tui my_project_setup.md`.
