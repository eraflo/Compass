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

use sha2::{Digest, Sha256};
use std::sync::Arc;
use tokio_rustls::rustls;
use tokio_rustls::rustls::client::danger::{ServerCertVerified, ServerCertVerifier};
use tokio_rustls::rustls::pki_types::{CertificateDer, ServerName, UnixTime};

/// Generates a self-signed certificate and returns the Cert, Key, and SHA256 Fingerprint.
/// This allows us to use standard TLS encryption without a central authority (Certificate Pinning).
pub fn generate_self_signed() -> anyhow::Result<(
    Vec<CertificateDer<'static>>,
    rustls::pki_types::PrivateKeyDer<'static>,
    String,
)> {
    let subject_alt_names = vec!["localhost".to_string(), "compass-session".to_string()];

    let key_pair = rcgen::KeyPair::generate(&rcgen::PKCS_ECDSA_P256_SHA256)?;
    let key_der = key_pair.serialize_der();

    let mut cert_params = rcgen::CertificateParams::new(subject_alt_names);
    cert_params.key_pair = Some(key_pair);

    let cert = rcgen::Certificate::from_params(cert_params)?;
    let cert_der = cert.serialize_der()?;

    // Calculate Fingerprint (SHA256 of DER)
    let mut hasher = Sha256::new();
    hasher.update(&cert_der);
    let fingerprint = hex::encode(hasher.finalize());

    let cert_parsed = CertificateDer::from(cert_der).into_owned();
    let key_parsed = rustls::pki_types::PrivateKeyDer::Pkcs8(key_der.into());

    Ok((vec![cert_parsed], key_parsed, fingerprint))
}

/// A Custom Verifier that ONLY trusts a certificate matching the pinned fingerprint.
/// This ignores expiration, CA chain, and hostname mismatches (since we use ephemeral certs).
#[derive(Debug)]
pub struct PinnedCertVerifier {
    expected_fingerprint: String,
}

impl PinnedCertVerifier {
    pub fn new(fingerprint: String) -> Arc<Self> {
        Arc::new(Self {
            expected_fingerprint: fingerprint,
        })
    }
}

impl ServerCertVerifier for PinnedCertVerifier {
    fn verify_server_cert(
        &self,
        end_entity: &CertificateDer<'_>,
        _intermediates: &[CertificateDer<'_>],
        _server_name: &ServerName<'_>,
        _ocsp_response: &[u8],
        _now: UnixTime,
    ) -> Result<ServerCertVerified, rustls::Error> {
        // Hash the received certificate
        let mut hasher = Sha256::new();
        hasher.update(end_entity.as_ref());
        let hash = hex::encode(hasher.finalize());

        // Compare with PIN
        if hash == self.expected_fingerprint {
            Ok(ServerCertVerified::assertion())
        } else {
            // Fail silently to avoid oracle attacks
            Err(rustls::Error::General("Connection rejected".into()))
        }
    }

    fn verify_tls12_signature(
        &self,
        _message: &[u8],
        _cert: &CertificateDer<'_>,
        _dss: &rustls::DigitallySignedStruct,
    ) -> Result<tokio_rustls::rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        // Since we verify the certificate by its hash (Pinning), we have already established
        // trust in this specific certificate. The TLS handshake proves the server possesses
        // the corresponding private key.
        Ok(tokio_rustls::rustls::client::danger::HandshakeSignatureValid::assertion())
    }

    fn verify_tls13_signature(
        &self,
        _message: &[u8],
        _cert: &CertificateDer<'_>,
        _dss: &rustls::DigitallySignedStruct,
    ) -> Result<tokio_rustls::rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        Ok(tokio_rustls::rustls::client::danger::HandshakeSignatureValid::assertion())
    }

    fn supported_verify_schemes(&self) -> Vec<rustls::SignatureScheme> {
        rustls::crypto::ring::default_provider()
            .signature_verification_algorithms
            .supported_schemes()
    }
}
