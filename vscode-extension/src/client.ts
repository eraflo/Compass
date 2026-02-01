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

import * as cp from 'child_process';
import * as readline from 'readline';
import * as vscode from 'vscode';

/**
 * Represents a single step in the runbook.
 */
export interface Step {
    title: string;
    status: string;
    description: string;
}

/**
 * Handles communication with the Compass CLI in headless mode via JSON-RPC.
 */
export class CompassClient {
    private process?: cp.ChildProcess;
    private reader?: readline.Interface;
    private rpcId = 1;
    private pendingRequests = new Map<number, (res: any) => void>();

    // Notification Emitter for streaming logs
    private _onLog = new vscode.EventEmitter<string>();
    readonly onLog = this._onLog.event;

    constructor(private filePath: string) {}

    /**
     * Starts the Compass CLI process in headless mode.
     */
    start(): Promise<void> {
        return new Promise((resolve, reject) => {
            const config = vscode.workspace.getConfiguration('compass');
            const binaryPath = config.get<string>('binaryPath') || 'compass';

            this.process = cp.spawn(binaryPath, ['--headless', this.filePath], {
                stdio: ['pipe', 'pipe', 'pipe']
            });

            this.process.on('error', (err) => {
                reject(new Error(`Failed to spawn 'compass'. Is it installed? ${err.message}`));
            });

            if (!this.process.stdout) {
                reject(new Error("No stdout"));
                return;
            }

            this.process.stderr?.on('data', (data) => {
                console.log(`[Compass Stderr]: ${data}`);
            });

            this.reader = readline.createInterface({
                input: this.process.stdout,
                terminal: false
            });

            this.reader.on('line', (line) => {
                if (!line.trim()) return;
                try {
                    const json = JSON.parse(line);

                    // Handle Notification
                    if (!json.id && json.method === 'log' && json.params?.output) {
                        this._onLog.fire(json.params.output);
                        return;
                    }

                    // Handle Response
                    if (json.id && this.pendingRequests.has(json.id)) {
                        this.pendingRequests.get(json.id)!(json);
                        this.pendingRequests.delete(json.id);
                    }
                } catch (e) {
                    console.error("Invalid JSON from Compass:", line);
                }
            });

            // Give it a brief moment to potentially crash or exit
            setTimeout(() => {
                if (this.process?.exitCode !== null) {
                    reject(new Error(`Compass exited immediately with code ${this.process?.exitCode}`));
                } else {
                    resolve();
                }
            }, 500);
        });
    }

    sendRequest(method: string, params?: any): Promise<any> {
        return new Promise((resolve, reject) => {
            if (!this.process) return reject("Process not started");

            const id = this.rpcId++;
            const req = { jsonrpc: '2.0', method, params, id };
            
            try {
                this.process.stdin?.write(JSON.stringify(req) + '\n');
            } catch (e) {
                reject(e);
                return;
            }

            // Set timeout
            const timeout = setTimeout(() => {
                if (this.pendingRequests.has(id)) {
                    this.pendingRequests.delete(id);
                    reject(new Error("Request timed out"));
                }
            }, 5000);

            this.pendingRequests.set(id, (response) => {
                clearTimeout(timeout);
                if (response.error) {
                    reject(new Error(response.error.message));
                } else {
                    resolve(response.result);
                }
            });
        });
    }

    async getSteps(): Promise<Step[]> {
        return this.sendRequest('get_steps');
    }

    async executeStep(index: number): Promise<void> {
        return this.sendRequest('execute_step', { index });
    }

    dispose() {
        if (this.process) {
            this.process.kill();
            this.process = undefined;
        }
    }
}
