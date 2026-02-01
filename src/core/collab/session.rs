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

use crate::core::collab::events::CompassEvent;
use std::sync::mpsc::Receiver;
use tokio::sync::mpsc::UnboundedSender;

/// State for the active collaboration session.
pub struct CollabSession {
    /// Is this instance the host?
    pub is_host: bool,
    /// The session ID (e.g. "localhost:3030")
    pub id: Option<String>,
    /// Channel to emit events to the network layer (Host only)
    pub tx: Option<UnboundedSender<CompassEvent>>,
    /// Channel to receive events from the network layer (Guest only)
    pub rx: Option<Receiver<CompassEvent>>,
}

impl CollabSession {
    pub fn new(
        is_host: bool,
        id: Option<String>,
        tx: Option<UnboundedSender<CompassEvent>>,
        rx: Option<Receiver<CompassEvent>>,
    ) -> Self {
        Self {
            is_host,
            id,
            tx,
            rx,
        }
    }
}
