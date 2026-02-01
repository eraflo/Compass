/*
 * Copyright 2026 eraflo
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

import * as vscode from 'vscode';
import * as cp from 'child_process';
import { CompassClient } from './client';
import { StepsProvider, StepItem } from './stepsProvider';

let client: CompassClient | undefined;
// Track the active CLI process (search, check, etc.) to forward input
let activeProcess: cp.ChildProcess | undefined;
// Use a Pseudoterminal for colored output
let writeEmitter: vscode.EventEmitter<string>;
let terminal: vscode.Terminal;

function getBinaryPath(): string {
    const config = vscode.workspace.getConfiguration('compass');
    return process.env.COMPASS_BINARY_PATH || config.get<string>('binaryPath') || 'compass';
}

function runInCompassTerminal(args: string[]) {
    if (!terminal) return;
    terminal.show(true);
    const binary = getBinaryPath();
    
    writeEmitter.fire(`\x1b[33m$ compass ${args.join(' ')}\x1b[0m\r\n`);

    const child = cp.spawn(binary, args, { shell: true });
    activeProcess = child;

    child.stdout.on('data', (data) => {
        const formatted = data.toString().replace(/\n/g, '\r\n');
        writeEmitter.fire(formatted);
    });

    child.stderr.on('data', (data) => {
        const formatted = data.toString().replace(/\n/g, '\r\n');
        writeEmitter.fire(`\x1b[31m${formatted}\x1b[0m`);
    });

    child.on('error', (err) => {
         writeEmitter.fire(`\x1b[31mFailed to start subprocess: ${err.message}\x1b[0m\r\n`);
    });

    child.on('close', (code) => {
        writeEmitter.fire(`\r\n\x1b[90mCommand exited with code ${code}\x1b[0m\r\n\r\n`);
        if (activeProcess === child) {
            activeProcess = undefined;
        }
    });
}

/**
 * Activates the Compass extension.
 */
export function activate(context: vscode.ExtensionContext) {
    // Setup Pseudoterminal
    writeEmitter = new vscode.EventEmitter<string>();
    const pty: vscode.Pseudoterminal = {
        onDidWrite: writeEmitter.event,
        open: () => {
             writeEmitter.fire('\x1b[36mCompass Navigator Terminal Ready\x1b[0m\r\n\r\n');
        },
        close: () => {
            if (activeProcess) {
                activeProcess.kill();
            }
        },
        handleInput: (data) => {
            if (activeProcess && !activeProcess.killed && activeProcess.stdin) {
                // Handle Ctrl+C (End of Text)
                if (data === '\x03') {
                    writeEmitter.fire('^C\r\n');
                    activeProcess.kill();
                    return;
                }
                
                // Simple Echo for feedback (optional, but good for CLI feel)
                if (data === '\r') {
                    writeEmitter.fire('\r\n');
                    activeProcess.stdin.write('\n');
                } else {
                    writeEmitter.fire(data);
                    activeProcess.stdin.write(data);
                }
            }
        }
    };
    terminal = vscode.window.createTerminal({ name: 'Compass Navigator', pty });
    context.subscriptions.push(terminal);


    // Initialize Tree Data Provider
    const stepsProvider = new StepsProvider();
    vscode.window.registerTreeDataProvider('compassSteps', stepsProvider);

    // Register command to start Compass on the current file
    const startCommand = vscode.commands.registerCommand('compass.start', async () => {
        const editor = vscode.window.activeTextEditor;
        if (!editor) {
            vscode.window.showErrorMessage("Open a Markdown file first!");
            return;
        }

        if (editor.document.languageId !== 'markdown') {
            vscode.window.showErrorMessage("Compass only works with Markdown files.");
            return;
        }

        // Dispose previous client if exists
        if (client) { client.dispose(); }

        const filePath = editor.document.fileName;
        client = new CompassClient(filePath);

        // Subscribe to real-time logs from the Rust backend
        client.onLog((data) => {
            // Normalize line endings for terminal
            const formatted = data.replace(/\n/g, '\r\n');
            writeEmitter.fire(formatted);
        });
        
        vscode.window.withProgress({
        location: vscode.ProgressLocation.Notification,
        title: "Starting Compass..."
        }, async () => {
            try {
                // Start the backend process
                await client!.start();
                // Fetch initial steps
                const steps = await client!.getSteps();
                stepsProvider.refresh(steps, client!);
                vscode.window.showInformationMessage(`Compass Navigator started.`);
            } catch (e: any) {
                vscode.window.showErrorMessage(`Compass Error: ${e.message}`);
                writeEmitter.fire(`\x1b[31mError: ${e.message}\x1b[0m\r\n`);
            }
        });
    });

    context.subscriptions.push(startCommand);

    // Auto-refresh when switching to a Markdown file
    context.subscriptions.push(
        vscode.window.onDidChangeActiveTextEditor(editor => {
            if (editor && editor.document.languageId === 'markdown') {
                vscode.commands.executeCommand('compass.start');
            }
        })
    );

    // Register command to execute a specific step
    context.subscriptions.push(
        vscode.commands.registerCommand('compass.executeStep', async (item: StepItem) => {
            if (!client) {
                vscode.window.showErrorMessage("Compass is not running.");
                return;
            }
            
            try {
                // Focus the terminal
                terminal.show(true);

                // Show step context (Titre + Description) nicely formatted
                writeEmitter.fire('\x1b[2J\x1b[3J\x1b[H'); // Clear screen
                writeEmitter.fire(`\x1b[1;36m### ${item.step.title} ###\x1b[0m\r\n\r\n`);
                
                if (item.step.description) {
                    writeEmitter.fire(`\x1b[37m${item.step.description.replace(/\n/g, '\r\n')}\x1b[0m\r\n\r\n`);
                }

                writeEmitter.fire(`\x1b[33mRunning Step ${item.index + 1}...\x1b[0m\r\n\r\n`);
                
                // Execute step via RPC
                await client.executeStep(item.index);
                
                // Refresh list to show new status (Success/Failed)
                const steps = await client.getSteps();
                stepsProvider.refresh(steps, client);
            } catch (e: any) {
                vscode.window.showErrorMessage(`Execution Failed: ${e.message}`);
                writeEmitter.fire(`\x1b[31mExecution Failed: ${e.message}\x1b[0m\r\n`);
            }
        })
    );

    // Register Registry Search
    context.subscriptions.push(
        vscode.commands.registerCommand('compass.search', async () => {
            const query = await vscode.window.showInputBox({ 
                placeHolder: "Search for runbooks (e.g., 'onboarding', 'rust', 'deploy')...",
                title: "Compass Registry Search"
            });
            if (query) {
                runInCompassTerminal(['search', query]);
            }
        })
    );

    // Register Check Integrity
    context.subscriptions.push(
        vscode.commands.registerCommand('compass.check', async () => {
            const editor = vscode.window.activeTextEditor;
            if (editor && editor.document.languageId === 'markdown') {
                runInCompassTerminal(['check', editor.document.fileName]);
            } else {
                vscode.window.showInformationMessage("Checking system health...");
                runInCompassTerminal(['check', 'system']);
            }
        })
    );
}

/**
 * Deactivates the extension.
 * Cleans up the Compass client process.
 */
export function deactivate() {
    if (client) {
        client.dispose();
    }
}
