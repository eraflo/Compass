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

use super::events::CompassEvent;
use futures_util::StreamExt;
use std::sync::Arc;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;

/// Starts the Guest Client.
///
/// Connects to `url` securely using Certificate Pinning.
pub async fn start_guest_client(
    url: String,
    app_tx: std::sync::mpsc::Sender<CompassEvent>,
) -> anyhow::Result<()> {
    // Parse URL and extract PIN
    let parsed_url = url::Url::parse(&url)?;
    let pin = parsed_url
        .query_pairs()
        .find(|(k, _)| k == "pin")
        .ok_or_else(|| {
            anyhow::anyhow!(
                "Missing ?pin= parameter. Secure connection requires certificate fingerprint."
            )
        })?
        .1
        .to_string();

    // 1. Setup TLS Config with Pinning
    let _root_store = tokio_rustls::rustls::RootCertStore::empty();
    // We don't need system roots because we only trust the pinned cert

    let verifier = super::security::PinnedCertVerifier::new(pin.clone());

    let config = tokio_rustls::rustls::ClientConfig::builder()
        .dangerous() // Explicit opt-in to custom verifier
        .with_custom_certificate_verifier(verifier)
        .with_no_client_auth();

    let config = Arc::new(config);
    let connector = tokio_tungstenite::Connector::Rustls(config);

    // 2. Prepare Request with Auth Header
    let mut request = url.into_client_request()?;
    request.headers_mut().insert("x-compass-pin", pin.parse()?);

    // 3. Connect
    let (ws_stream, _) = match tokio_tungstenite::connect_async_tls_with_config(
        request,
        None,
        false,
        Some(connector),
    )
    .await
    {
        Ok(v) => v,
        Err(e) => {
            eprintln!(
                "🔥 Security Alert: Connection rejected. The server's certificate did NOT match the pinned fingerprint."
            );
            eprintln!("   This could mean a Man-In-The-Middle attack, or the session ID is wrong.");
            anyhow::bail!("TLS Handshake Error: {}", e);
        }
    };

    println!("✅ Securely connected to Host.");

    let (_, mut read) = ws_stream.split();

    while let Some(msg) = read.next().await {
        match msg {
            Ok(tokio_tungstenite::tungstenite::Message::Text(text)) => {
                if let Ok(event) = serde_json::from_str::<CompassEvent>(&text) {
                    let _ = app_tx.send(event);
                }
            }
            Ok(tokio_tungstenite::tungstenite::Message::Close(_)) => {
                let _ = app_tx.send(CompassEvent::ConnectionLost(
                    "Host closed connection.".to_string(),
                ));
                break;
            }
            Err(e) => {
                let _ = app_tx.send(CompassEvent::ConnectionLost(format!(
                    "Connection error: {}",
                    e
                )));
                break;
            }
            _ => {}
        }
    }

    Ok(())
}
