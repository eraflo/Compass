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
import { CompassClient } from './client';
import { StepsProvider, StepItem } from './stepsProvider';

let client: CompassClient | undefined;
let outputChannel: vscode.OutputChannel;

/**
 * Activates the Compass extension.
 * This function is called when the extension is activated (e.g., when a Markdown file is opened).
 * 
 * @param context - The extension context provided by VS Code.
 */
export function activate(context: vscode.ExtensionContext) {
    // Create Output Channel for streaming logs
    outputChannel = vscode.window.createOutputChannel("Compass Navigator");
    context.subscriptions.push(outputChannel);

    // Initialize Tree Data Provider
    const stepsProvider = new StepsProvider();
    vscode.window.registerTreeDataProvider('compassSteps', stepsProvider);

    // Register command to start Compass on the current file
    context.subscriptions.push(
        vscode.commands.registerCommand('compass.start', async () => {
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
                outputChannel.append(data);
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
                    outputChannel.appendLine(`Error: ${e.message}`);
                }
            });
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
                // Focus and clear output channel before running
                outputChannel.clear();
                outputChannel.show(true);
                outputChannel.appendLine(`Running Step ${item.index + 1}: ${item.step.title}\n`);
                
                // Execute step via RPC
                await client.executeStep(item.index);
                
                // Refresh list to show new status (Success/Failed)
                const steps = await client.getSteps();
                stepsProvider.refresh(steps, client);
            } catch (e: any) {
                vscode.window.showErrorMessage(`Execution Failed: ${e.message}`);
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
