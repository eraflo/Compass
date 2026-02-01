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
import { CompassClient, Step } from './client';

/**
 * TreeDataProvider implementation for the Compass Steps view.
 * Displays the list of steps and their execution status.
 */
export class StepsProvider implements vscode.TreeDataProvider<StepItem> {
    private _onDidChangeTreeData: vscode.EventEmitter<StepItem | undefined | null | void> = new vscode.EventEmitter<StepItem | undefined | null | void>();
    readonly onDidChangeTreeData: vscode.Event<StepItem | undefined | null | void> = this._onDidChangeTreeData.event;

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

    getTreeItem(element: StepItem): vscode.TreeItem {
        return element;
    }

    getChildren(element?: StepItem): Thenable<StepItem[]> {
        if (element) {
            return Promise.resolve([]);
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
        super(`${index + 1}. ${step.title}`, vscode.TreeItemCollapsibleState.None);
        
        this.description = step.status;
        this.tooltip = step.description;
        
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

        // Click to run
        this.command = {
            command: 'compass.executeStep',
            title: 'Run Step',
            arguments: [this]
        };
    }
}
