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

document.addEventListener('DOMContentLoaded', () => {
    const grid = document.getElementById('grid-container');
    const searchInput = document.getElementById('search-input');
    let allRunbooks = [];

    // Fetch the registry JSON
    fetch('./registry.json')
        .then(response => {
            if (!response.ok) throw new Error("Failed to load registry");
            return response.json();
        })
        .then(data => {
            allRunbooks = data;
            renderRunbooks(allRunbooks);
        })
        .catch(err => {
            grid.innerHTML = `<div class="loading" style="color: #ff4b4b;">Error: ${err.message}</div>`;
        });

    // Render function
    function renderRunbooks(items) {
        grid.innerHTML = '';
        
        if (items.length === 0) {
            grid.innerHTML = '<div class="loading">No runbooks found.</div>';
            return;
        }

        items.forEach(book => {
            const card = document.createElement('div');
            card.className = 'card';
            
            // Extract owner/repo for display
            const repoUrl = book.url.replace('.git', '');
            const repoName = repoUrl.split('/').slice(-2).join('/');
            
            // Clone command
            const cloneCmd = `compass clone ${book.url}`;

            card.innerHTML = `
                <div class="card-header">
                    <div>
                        <h3>${book.name}</h3>
                        <div class="author">by ${book.author || 'Community'}</div>
                    </div>
                </div>
                <p class="description">${book.description}</p>
                
                <div class="snippet">
                    <code>${cloneCmd}</code>
                    <button class="copy-btn" onclick="copyToClipboard('${cloneCmd}', this)" title="Copy command">
                        <svg viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="2"><rect x="9" y="9" width="13" height="13" rx="2" ry="2"></rect><path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"></path></svg>
                    </button>
                </div>

                <div class="tags">
                    ${(book.tags || []).map(tag => `<span class="tag">#${tag}</span>`).join('')}
                    <span class="tag" style="margin-left:auto; opacity:0.6;">📦 v${book.version || '1.0'}</span>
                </div>
            `;
            grid.appendChild(card);
        });
    }

    // Search Logic
    searchInput.addEventListener('input', (e) => {
        const query = e.target.value.toLowerCase();
        const filtered = allRunbooks.filter(book => 
            book.name.toLowerCase().includes(query) ||
            book.description.toLowerCase().includes(query) ||
            (book.tags && book.tags.some(t => t.toLowerCase().includes(query)))
        );
        renderRunbooks(filtered);
    });
});

// Clipboard Utility
function copyToClipboard(text, btn) {
    navigator.clipboard.writeText(text).then(() => {
        const originalIcon = btn.innerHTML;
        btn.innerHTML = '<svg viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="#4ade80" stroke-width="2"><polyline points="20 6 9 17 4 12"></polyline></svg>';
        setTimeout(() => {
            btn.innerHTML = originalIcon;
        }, 2000);
    });
}
