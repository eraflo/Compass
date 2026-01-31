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

use super::UrlRewriter;
use url::Url;

/// Rewriter for GitHub URLs.
/// Converts `github.com/.../blob/...` to `raw.githubusercontent.com/...`
pub struct GitHubRewriter;

impl UrlRewriter for GitHubRewriter {
    fn can_handle(&self, url: &Url) -> bool {
        url.host_str() == Some("github.com")
    }

    fn rewrite(&self, url: &Url) -> Option<Url> {
        let path = url.path();
        if path.contains("/blob/") {
            let new_path = path.replace("/blob/", "/");
            let mut new_url = url.clone();
            if new_url.set_host(Some("raw.githubusercontent.com")).is_ok() {
                new_url.set_path(&new_path);
                return Some(new_url);
            }
        }
        None
    }
}
