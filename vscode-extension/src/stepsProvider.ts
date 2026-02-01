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
import { CompassClient, Step, CodeBlock } from './client';

/**
 * TreeDataProvider implementation for the Compass Steps view.
 * Displays the list of steps and their execution status.
 */
export class StepsProvider implements vscode.TreeDataProvider<StepItem | CodeItem> {
    private _onDidChangeTreeData: vscode.EventEmitter<StepItem | CodeItem | undefined | null | void> = new vscode.EventEmitter<StepItem | CodeItem | undefined | null | void>();
    readonly onDidChangeTreeData: vscode.Event<StepItem | CodeItem | undefined | null | void> = this._onDidChangeTreeData.event;

    private steps: Step[] = [];
    private client?: CompassClient;

    /**
     * Updates the tree view with new data.
     * @param steps - The list of steps to display.
     * @param client - The active Compass client.
     */
    refresh(steps: Step[], client: CompassClient): void {
        this.steps = steps;
        this.client = client;
        this._onDidChangeTreeData.fire();
    }

    getTreeItem(element: StepItem | CodeItem): vscode.TreeItem {
        return element;
    }

    getChildren(element?: StepItem): Thenable<StepItem[] | CodeItem[]> {
        if (element) {
            // Return code blocks for this step
            return Promise.resolve(
                element.step.code_blocks.map((block, index) => new CodeItem(block, index))
            );
        }
        return Promise.resolve(
            this.steps.map((step, index) => new StepItem(step, index))
        );
    }
}

/**
 * Represents a single item in the steps tree view.
 */
export class StepItem extends vscode.TreeItem {
    constructor(
        public readonly step: Step,
        public readonly index: number
    ) {
        // Collapsible only if it has code blocks
        const state = step.code_blocks.length > 0
            ? vscode.TreeItemCollapsibleState.Collapsed
            : vscode.TreeItemCollapsibleState.None;

        super(`${index + 1}. ${step.title}`, state);
        
        this.description = step.status;
        this.tooltip = step.description || "No description";
        
        // Icon logic
        if (step.status === 'Success') {
            this.iconPath = new vscode.ThemeIcon('check');
        } else if (step.status === 'Failed') {
            this.iconPath = new vscode.ThemeIcon('error');
        } else if (step.status === 'Running') {
             this.iconPath = new vscode.ThemeIcon('loading~spin');
        } else {
            this.iconPath = new vscode.ThemeIcon('circle-outline');
        }

        // Context value for commands
        this.contextValue = 'step';
        
        // If it's a leaf (no code blocks), make it clickable to run
        if (state === vscode.TreeItemCollapsibleState.None) {
            this.command = {
                command: 'compass.executeStep',
                title: 'Run Step',
                arguments: [this]
            };
        }
    }
}

export class CodeItem extends vscode.TreeItem {
    constructor(
        public readonly block: CodeBlock,
        public readonly index: number
    ) {
        // Show first line or truncation
        const summary = block.content.split('\n')[0].trim();
        super(summary || "(empty line)", vscode.TreeItemCollapsibleState.None);

        this.description = block.language || "text";
        this.tooltip = new vscode.MarkdownString();
        this.tooltip.appendCodeblock(block.content, block.language || 'text');
        
        this.iconPath = new vscode.ThemeIcon('code');
    }
}
