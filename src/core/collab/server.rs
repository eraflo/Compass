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
use futures_util::{SinkExt, StreamExt};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::broadcast;
use tokio::sync::mpsc::UnboundedReceiver;
use tokio_rustls::TlsAcceptor;

/// Starts the Host Server.
///
/// Listens on `0.0.0.0:3030`.
/// - Uses self-signed TLS Certificate.
/// - Uses PIN (Certificate Fingerprint) for authentication.
pub async fn start_host_server(
    mut app_rx: UnboundedReceiver<CompassEvent>,
    certs: Vec<tokio_rustls::rustls::pki_types::CertificateDer<'static>>,
    key: tokio_rustls::rustls::pki_types::PrivateKeyDer<'static>,
    pin: String,
) -> anyhow::Result<()> {
    // 2. Setup TLS Config
    let tls_config = tokio_rustls::rustls::ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(certs, key)?;
    let acceptor = TlsAcceptor::from(Arc::new(tls_config));

    let port = 3030;

    // 3. Setup Broadcast Channel
    let (broadcast_tx, _) = broadcast::channel::<String>(100);

    // State Cache for new joiners
    let last_snapshot = Arc::new(std::sync::RwLock::new(None::<String>));

    // 4. Spawn Event Broadcaster
    let b_tx = broadcast_tx.clone();
    let cache_writer = last_snapshot.clone();

    tokio::spawn(async move {
        while let Some(event) = app_rx.recv().await {
            // Cache snapshot if valid
            if let CompassEvent::Snapshot { .. } = &event
                && let Ok(json) = serde_json::to_string(&event)
                && let Ok(mut writer) = cache_writer.write()
            {
                *writer = Some(json.clone());
            }

            if let Ok(json) = serde_json::to_string(&event) {
                let _ = b_tx.send(json);
            }
        }
    });

    // 5. Listen for Connections
    let listener = TcpListener::bind(format!("0.0.0.0:{}", port)).await?;

    while let Ok((stream, addr)) = listener.accept().await {
        let b_rx = broadcast_tx.subscribe();
        let acceptor_clone = acceptor.clone();
        let pin_clone = pin.clone();
        let cache_reader = last_snapshot.clone();

        tokio::spawn(async move {
            match acceptor_clone.accept(stream).await {
                Ok(tls_stream) => {
                    // Upgrade to WebSocket over TLS
                    if let Err(_e) =
                        handle_connection(tls_stream, addr, b_rx, pin_clone, cache_reader).await
                    {
                        // Connection failed, usually client disconnect or handshake error
                    }
                }
                Err(e) => {
                    eprintln!("TLS Handshake failed from {}: {}", addr, e);
                }
            }
        });
    }

    Ok(())
}

/// Handles a single guest connection (already wrapped in TLS).
/// Note: We strictly use the "websocket" stream which abstracts over TlsStream.
async fn handle_connection(
    stream: tokio_rustls::server::TlsStream<TcpStream>,
    addr: SocketAddr,
    mut b_rx: broadcast::Receiver<String>,
    expected_pin: String,
    initial_state_cache: Arc<std::sync::RwLock<Option<String>>>,
) -> anyhow::Result<()> {
    // Explicitly verify the client knows the PIN.
    // This prevents unauthorized connections from just ignoring cert errors.
    let callback =
        |req: &tokio_tungstenite::tungstenite::handshake::server::Request,
         response: tokio_tungstenite::tungstenite::handshake::server::Response| {
            if let Some(header_value) = req.headers().get("x-compass-pin")
                && let Ok(val) = header_value.to_str()
                && val == expected_pin
            {
                return Ok(response);
            }
            Err(
                tokio_tungstenite::tungstenite::handshake::server::ErrorResponse::new(Some(
                    "Unauthorized: Invalid or Missing PIN".to_string(),
                )),
            )
        };

    let ws_stream = tokio_tungstenite::accept_hdr_async(stream, callback).await?;
    // println!("✨ Guest connected (Secure + Authenticated): {}", addr); // Disabled to prevent TUI pollution

    let (mut write, mut read) = ws_stream.split();

    // Send immediate snapshot if available
    {
        let snapshot_opt = if let Ok(reader) = initial_state_cache.read() {
            reader.clone()
        } else {
            None
        };

        if let Some(json) = snapshot_opt {
            write
                .send(tokio_tungstenite::tungstenite::Message::Text(json.into()))
                .await?;
        }
    }

    loop {
        tokio::select! {
            msg = b_rx.recv() => {
                match msg {
                    Ok(json) => {
                        write.send(tokio_tungstenite::tungstenite::Message::Text(json.into())).await?;
                    }
                    Err(broadcast::error::RecvError::Lagged(_)) => {}
                    Err(broadcast::error::RecvError::Closed) => break,
                }
            }
            msg = read.next() => {
                match msg {
                    Some(Ok(tokio_tungstenite::tungstenite::Message::Close(_))) => break,
                    Some(Ok(tokio_tungstenite::tungstenite::Message::Ping(data))) => {
                        write.send(tokio_tungstenite::tungstenite::Message::Pong(data)).await?;
                    }
                    Some(Ok(_)) => {},
                    Some(Err(_)) => break,
                    None => break,
                }
            }
        }
    }

    println!("👋 Guest disconnected: {}", addr);
    Ok(())
}
