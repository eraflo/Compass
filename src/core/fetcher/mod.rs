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

pub mod rewriters;

use anyhow::{Context, Result, bail};
use reqwest::header::USER_AGENT;
use url::Url;

use self::rewriters::normalize_git_forge_url;

/// Fetches remote content from a URL.
/// Handles automatic conversion of GitHub/GitLab blob URLs to raw URLs.
pub fn fetch_remote_content(input_url: &str) -> Result<String> {
    let url = Url::parse(input_url).context("Invalid URL format")?;

    // Normalize URL for raw content if hosted on known forges (Moved to submodule)
    let target_url = normalize_git_forge_url(&url);
    let url_str = target_url.as_str();

    // Use version from Cargo.toml
    let current_version = env!("CARGO_PKG_VERSION");
    let user_agent = format!("Compass/{}", current_version);

    let client = reqwest::blocking::Client::new();
    let response = client
        .get(url_str)
        .header(USER_AGENT, user_agent)
        .send()
        .with_context(|| format!("Failed to connect to {}", url_str))?;

    if !response.status().is_success() {
        bail!(
            "Failed to download content. Status: {} - {}",
            response.status(),
            url_str
        );
    }

    // Optional: Check Content-Type to warn if it doesn't look like text
    if let Some(content_type) = response.headers().get(reqwest::header::CONTENT_TYPE) {
        let ct = content_type.to_str().unwrap_or("");
        if !ct.contains("text") && !ct.contains("markdown") && !ct.contains("plain") {
            // Low-level logging could be added here
        }
    }

    let content = response
        .text()
        .with_context(|| "Failed to read response body as text")?;

    Ok(content)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fetch_invalid_url() {
        assert!(fetch_remote_content("not-a-url").is_err());
    }
}
