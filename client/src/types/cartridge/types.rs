use serde::{Deserialize, Deserializer, Serialize, Serializer};
use starknet_rust_core::types::Felt;
use std::fmt;
use std::str::FromStr;
use url::Url;

#[derive(Debug, Clone, Deserialize)]
pub struct SessionAuthPrompt {
    pub authorization_url: Url,
    pub short_url: Url,
    pub public_key: Felt,
    pub expires: String,
    pub message: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SessionAuthSuccess {
    chain_id: String,
    message: String,
    public_key: Felt,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SessionStatus {
    pub address: Felt,
    pub chain_id: String,
    pub expires_at: i64,
    pub expires_at_formatted: String,
    pub expires_in_seconds: i64,
    pub guid: String,
    pub is_expired: bool,
    pub policies: Vec<ContractPolicyMethod>,
    pub public_key: Felt,
}

/// The `data` object of `session status`: `{ "session": {...} }` when a session
/// exists, `{ "session": null }` when it doesn't.
#[derive(Debug, Clone, Deserialize)]
pub struct SessionStatusData {
    pub session: Option<SessionStatus>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContractPolicyMethod {
    pub method: String,
    pub address: Felt,
}

impl FromStr for ContractPolicyMethod {
    type Err = String;

    /// Parses `"<address-hex>:<method>"`, e.g. `"0x6ae3…:attack"`.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // The address hex contains no ':', so the first colon is the separator.
        let (address, method) = s
            .split_once(':')
            .ok_or_else(|| format!("expected '<address>:<method>', got '{s}'"))?;
        if method.is_empty() {
            return Err(format!("missing method in policy '{s}'"));
        }
        let address = Felt::from_hex(address)
            .map_err(|e| format!("invalid policy address '{address}': {e}"))?;
        Ok(ContractPolicyMethod {
            address,
            method: method.to_string(),
        })
    }
}

impl fmt::Display for ContractPolicyMethod {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:#x}:{}", self.address, self.method)
    }
}

impl<'de> Deserialize<'de> for ContractPolicyMethod {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        s.parse().map_err(serde::de::Error::custom)
    }
}



#[derive(Clone, Serialize)]
pub struct PolicyMethod {
    pub name: &'static str,
    pub entrypoint: &'static str,
    pub description: &'static str,
}

impl PolicyMethod {
    pub const fn new(
        name: &'static str,
        entrypoint: &'static str,
        description: &'static str,
    ) -> Self {
        Self {
            name,
            entrypoint,
            description,
        }
    }
}