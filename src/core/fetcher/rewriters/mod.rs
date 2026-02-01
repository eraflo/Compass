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

pub mod github;
pub mod gitlab;

use url::Url;

use self::github::GitHubRewriter;
use self::gitlab::GitLabRewriter;

/// Trait to define a URL rewriter strategy for specific hosts.
pub trait UrlRewriter {
    /// Determines if this rewriter supports the given URL.
    fn can_handle(&self, url: &Url) -> bool;

    /// Rewrites the URL to a raw content URL if applicable.
    /// Returns None if no rewriting is needed/possible despite handling the host.
    fn rewrite(&self, url: &Url) -> Option<Url>;
}

/// Main entry point to normalize URLs using registered rewriters.
pub fn normalize_git_forge_url(url: &Url) -> Url {
    // List of available rewriters
    // In a larger system, this could be dynamic or plugin-based.
    let rewriters: Vec<Box<dyn UrlRewriter>> =
        vec![Box::new(GitHubRewriter), Box::new(GitLabRewriter)];

    for rewriter in rewriters {
        if rewriter.can_handle(url)
            && let Some(rewritten) = rewriter.rewrite(url)
        {
            return rewritten;
        }
    }

    // Default: return original URL
    url.clone()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_github_normalization() {
        let input = "https://github.com/user/repo/blob/main/README.md";
        let url = Url::parse(input).unwrap();
        let normalized = normalize_git_forge_url(&url);
        assert_eq!(
            normalized.as_str(),
            "https://raw.githubusercontent.com/user/repo/main/README.md"
        );
    }

    #[test]
    fn test_gitlab_normalization() {
        let input = "https://gitlab.com/user/repo/-/blob/main/README.md";
        let url = Url::parse(input).unwrap();
        let normalized = normalize_git_forge_url(&url);
        assert_eq!(
            normalized.as_str(),
            "https://gitlab.com/user/repo/-/raw/main/README.md"
        );
    }

    #[test]
    fn test_no_normalization() {
        let input = "https://example.com/README.md";
        let url = Url::parse(input).unwrap();
        let normalized = normalize_git_forge_url(&url);
        assert_eq!(normalized.as_str(), input);
    }
}
