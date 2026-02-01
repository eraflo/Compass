---
name: license-header
description: Checks and adds Apache 2.0 license headers to the beginning of source files.
---

# License Header Skill

This skill ensures that all source files (Rust, Javascript, etc.) have the standard Apache 2.0 license header at the top.

## Header Template

```rust
// Copyright 2026 eraflo
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
```

## Usage

When creating a new file or modifying an existing one, ensure the header is present.

### For Rust Files (`.rs`)
Use the `//` comment syntax.

### For Markdown Files (`.md`)
Usually headers are not required for Markdown files, but if needed, use HTML comments `<!-- ... -->`.

## Automated Script
You can use the following PowerShell script to check and add the header if missing.

```powershell
$header = @'
// Copyright 2026 eraflo
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
'@

$files = Get-ChildItem -Path src -Filter *.rs -Recurse
foreach ($file in $files) {
    $content = Get-Content $file.FullName -Raw
    if (-not $content.StartsWith("// Copyright")) {
        Write-Host "Adding header to $($file.Name)"
        $newContent = $header + "`r`n`r`n" + $content
        Set-Content -Path $file.FullName -Value $newContent
    }
}
```
