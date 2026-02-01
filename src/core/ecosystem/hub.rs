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

use anyhow::{Context, Result};
use serde::Deserialize;
use std::env;
use std::time::Duration;

const DEFAULT_REGISTRY_URL: &str = "https://eraflo.github.io/Compass/registry.json";

#[derive(Debug, Deserialize, Clone)]
pub struct RemoteRunbook {
    pub name: String,
    pub description: String,
    #[serde(default)]
    pub author: String,
    #[serde(default)]
    pub stars: u32,
    pub url: String,
    #[serde(default)]
    pub tags: Vec<String>,
}

/// Fetches the full registry.
async fn fetch_registry() -> Result<Vec<RemoteRunbook>> {
    let hub_url = env::var("COMPASS_HUB_URL").unwrap_or_else(|_| DEFAULT_REGISTRY_URL.to_string());

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .user_agent(concat!("Compass-CLI/", env!("CARGO_PKG_VERSION")))
        .build()?;

    let response = client
        .get(&hub_url)
        .send()
        .await
        .context("Failed to contact Compass Hub")?;

    if !response.status().is_success() {
        anyhow::bail!("Compass Hub returned error status: {}", response.status());
    }

    // Try parsing as simple array
    let packages: Vec<RemoteRunbook> = response
        .json()
        .await
        .context("Failed to parse registry JSON")?;

    Ok(packages)
}

/// Searches the remote Compass Hub (GitHub Registry) for runbooks matching the query.
pub async fn search_remote(query: &str) -> Result<Vec<RemoteRunbook>> {
    let packages = fetch_registry().await?;
    let query_lower = query.to_lowercase();

    let filtered = packages
        .into_iter()
        .filter(|p| {
            p.name.to_lowercase().contains(&query_lower)
                || p.description.to_lowercase().contains(&query_lower)
                || p.tags
                    .iter()
                    .any(|t| t.to_lowercase().contains(&query_lower))
        })
        .collect();

    Ok(filtered)
}

/// Resolves a single runbook by name (exact match).
pub async fn resolve_runbook(name: &str) -> Result<Option<RemoteRunbook>> {
    let packages = fetch_registry().await?;
    Ok(packages.into_iter().find(|p| p.name == name))
}
