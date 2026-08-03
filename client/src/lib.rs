// Public modules available to integration tests and external crates
pub mod game;
pub mod merkle;
pub mod types;
pub mod utils;

/// Installs the process-wide rustls crypto provider (aws-lc-rs).
///
/// rustls 0.23 is compiled with both `aws-lc-rs` (from this crate) and `ring`
/// (pulled in transitively by the starknet crates), so it cannot pick a default
/// provider on its own and panics the first time TLS is used. Call this once at
/// the start of every binary's `main`, before any HTTPS/WSS request. Idempotent:
/// a second call is a no-op.
pub fn install_crypto_provider() {
    let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();
}
