use crate::types::cartridge::types::{
    PolicyMethod, SessionAuthPrompt, SessionAuthSuccess, SessionStatus, SessionStatusData,
};
use crate::types::error::{CartridgeCliError, GameError};
use crate::types::result::Result;
use log::{error, info};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use starknet_rust_core::types::{BlockId, BlockTag, Call, Felt, FunctionCall};
use starknet_rust_core::utils::parse_cairo_short_string;
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub enum CliEvent {
    /// `{ "status": "info", "message": ... }` — progress/log line.
    Info { message: String },
    /// `{ "status": "success", "data": {...} }` — a success payload (the `data` object).
    ///
    /// Multi-step commands emit several of these: e.g. `register`/`session auth` emits
    /// one carrying `authorization_url`/`short_url` (the browser prompt) *before* the
    /// final one that arrives after approval.
    Success { data: Value },
    /// `{ "status": "error", "error_code", "message", "recovery_hint" }`.
    Error {
        code: String,
        message: String,
        hint: Option<String>,
    },
    /// Any other / unrecognized JSON line.
    Other(Value),
}

impl CliEvent {
    fn from_value(value: Value) -> Self {
        match value.get("status").and_then(Value::as_str) {
            Some("info") => CliEvent::Info {
                message: value
                    .get("message")
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_string(),
            },
            Some("success") => CliEvent::Success {
                // Payloads are nested under `data`; fall back to the whole object.
                data: value.get("data").cloned().unwrap_or(value),
            },
            Some("error") => CliEvent::Error {
                code: value
                    .get("error_code")
                    .and_then(Value::as_str)
                    .unwrap_or("Unknown")
                    .to_string(),
                message: value
                    .get("message")
                    .and_then(Value::as_str)
                    .unwrap_or("controller CLI returned an error")
                    .to_string(),
                hint: value
                    .get("recovery_hint")
                    .and_then(Value::as_str)
                    .filter(|h| !h.is_empty())
                    .map(str::to_string),
            },
            _ => CliEvent::Other(value),
        }
    }

    pub fn success_as<T: DeserializeOwned>(&self) -> Option<Result<T>> {
        match self {
            CliEvent::Success { data } => Some(serde_json::from_value(data.clone()).map_err(|e| {
                GameError::CartridgeCliError(CartridgeCliError::FailedToDeserialize(format!("{e}")))
            })),
            _ => None,
        }
    }

    /// Human-readable error text: `message`, with the recovery hint appended.
    fn error_string(message: &str, hint: Option<&str>) -> String {
        match hint {
            Some(hint) if !hint.is_empty() => format!("{message}\n↳ {hint}"),
            _ => message.to_string(),
        }
    }
}

#[derive(Serialize)]
struct CallFile {
    calls: Vec<CallSpec>,
}

#[derive(Serialize)]
struct CallSpec {
    #[serde(rename = "contractAddress")]
    contract_address: String,
    entrypoint: String,
    calldata: Vec<String>,
}

#[derive(Deserialize)]
struct ExecuteResult {
    #[serde(alias = "tx_hash", alias = "txHash")]
    transaction_hash: Felt,
}

/// `controller call --file` payload: one entry per requested call. The top-level
/// event stays `success` even when an individual call reverts, so `success`/`error`
/// must be inspected per call.
#[derive(Deserialize)]
struct CallResults {
    calls: Vec<CallOutcome>,
}

#[derive(Deserialize)]
struct CallOutcome {
    entrypoint: String,
    #[serde(default)]
    result: Vec<Felt>,
    success: bool,
    #[serde(default)]
    error: Option<String>,
}

pub struct CartridgeCLI {
    path: PathBuf,
    known_methods: HashMap<Felt, String>,
}

impl CartridgeCLI {
    pub fn new(
        path: impl Into<PathBuf>,
        known_methods: Vec<(Felt, String)>
    ) -> Self {
        Self {
            path: path.into(),
            known_methods: known_methods.into_iter().collect(),
        }
    }

    pub async fn auth(
        &self,
        contract_address: Felt,
        chain_id: &Felt,
        policies: Vec<PolicyMethod>
    ) -> Result<SessionAuthSuccess> {
        let json = Self::policies_json(contract_address, policies.clone());
        let temp_policies = Self::temp_file("policies", json).await?;

        let result: Result<SessionAuthSuccess> = self
            .run_cli_streaming(
                &[
                    "session",
                    "auth",
                    "--file",
                    temp_policies.to_string_lossy().as_ref(),
                    "--chain-id",
                    parse_cairo_short_string(chain_id).unwrap().as_str(),
                    "--overwrite",
                ],
                |event| {
                    // The browser prompt is a Success whose data matches SessionAuthPrompt;
                    // the final (post-approval) Success won't, so it's ignored here.
                    if let Some(Ok(prompt)) = event.success_as::<SessionAuthPrompt>() {
                        info!("Authorize in browser: {}", prompt.short_url);
                    } else if let CliEvent::Info { message } = &event {
                        info!("Cartridge CLI: {message}");
                    }
                },
            )
            .await;

        Self::remove_temp_file(temp_policies).await?;
        result
    }

    pub async fn status(&self) -> Result<SessionStatus> {
        let data: SessionStatusData = self.run_cli(&["session", "status"]).await?;

        data.session
            .ok_or(GameError::CartridgeCliError(CartridgeCliError::NoSession))
    }

    pub async fn username(&self) -> Result<String> {
        self.run_cli(&["username"]).await
    }

    pub async fn clear(&self) -> Result<()> {
        self.run_cli::<Value>(&["session", "clear"]).await?;
        Ok(())
    }

    /// `controller execute --file <calls>` against the active session. Uses the
    /// paymaster by default (gasless). Returns the submitted transaction hash.
    pub async fn execute(&self, calls: Vec<Call>) -> Result<Felt> {
        let specs = calls
            .iter()
            .map(|call| self.call_to_call_spec(call))
            .collect::<Result<Vec<_>>>()?;

        let json = serde_json::to_string(&CallFile { calls: specs }).map_err(|e| {
            GameError::CartridgeCliError(CartridgeCliError::CliError(format!(
                "Failed to serialize calls: {e}"
            )))
        })?;
        let temp_calls = Self::temp_file("calls", json).await?;

        let result: Result<ExecuteResult> = self
            .run_cli(&["execute", "--file", temp_calls.to_string_lossy().as_ref()])
            .await;

        Self::remove_temp_file(temp_calls).await?;
        Ok(result?.transaction_hash)
    }

    pub async fn call(
        &self,
        calls: Vec<FunctionCall>,
        block_id: BlockId,
    ) -> Result<Vec<Vec<Felt>>> {
        if let BlockId::Tag(tag) = block_id &&
            (tag == BlockTag::PreConfirmed || tag == BlockTag::L1Accepted)
        {
            return Err(GameError::CartridgeCliError(CartridgeCliError::CliError(
                format!("Block tag {:?} not supported", tag).to_string(),
            )));
        }

        let block_id_str = match block_id {
            BlockId::Hash(felt) => felt.to_fixed_hex_string(),
            BlockId::Number(num) => num.to_string(),
            BlockId::Tag(tag) => {
                if tag == BlockTag::PreConfirmed || tag == BlockTag::L1Accepted {
                    return Err(GameError::CartridgeCliError(CartridgeCliError::CliError(
                        format!("Block tag {:?} not supported", tag).to_string(),
                    )));
                }

                "latest".to_string()
            }
        };

        let specs = calls
            .iter()
            .map(|call| self.function_call_to_call_spec(call))
            .collect::<Result<Vec<_>>>()?;

        let json = serde_json::to_string(&CallFile { calls: specs }).map_err(|e| {
            GameError::CartridgeCliError(CartridgeCliError::CliError(format!(
                "Failed to serialize calls: {e}"
            )))
        })?;
        let temp_calls = Self::temp_file("calls", json).await?;

        let result: Result<CallResults> = self
            .run_cli(&["call", "--file", temp_calls.to_string_lossy().as_ref(), "--block-id", block_id_str.as_str()])
            .await;

        Self::remove_temp_file(temp_calls).await?;

        let outcomes = result?.calls;

        // A reverted call is reported per entry, not as a top-level error.
        if let Some(failed) = outcomes.iter().find(|c| !c.success) {
            let message = failed
                .error
                .clone()
                .unwrap_or_else(|| format!("call to `{}` reverted", failed.entrypoint));
            return Err(GameError::CartridgeCliError(CartridgeCliError::CliError(
                message,
            )));
        }

        Ok(outcomes.into_iter().map(|c| c.result).collect())
    }

    /// Converts a [`Call`] into the CLI's [`CallSpec`], resolving the selector
    /// back to its entrypoint name via `method_selectors` (filled at `auth`).
    fn call_to_call_spec(&self, call: &Call) -> Result<CallSpec> {
        let entrypoint = self.known_methods.get(&call.selector).ok_or_else(|| {
            GameError::CartridgeCliError(CartridgeCliError::CliError(format!(
                "no entrypoint name known for selector {}",
                call.selector.to_hex_string()
            )))
        })?;

        Ok(CallSpec {
            contract_address: call.to.to_hex_string(),
            entrypoint: entrypoint.clone(),
            calldata: call.calldata.iter().map(Felt::to_hex_string).collect(),
        })
    }

    /// Converts a [`FunctionCall`] into the CLI's [`CallSpec`], resolving the selector
    /// back to its entrypoint name via `method_selectors` (filled at `auth`).
    fn function_call_to_call_spec(&self, call: &FunctionCall) -> Result<CallSpec> {
        let entrypoint = self
            .known_methods
            .get(&call.entry_point_selector)
            .ok_or_else(|| {
                GameError::CartridgeCliError(CartridgeCliError::CliError(format!(
                    "no entrypoint name known for selector {}",
                    call.entry_point_selector.to_hex_string()
                )))
            })?;

        Ok(CallSpec {
            contract_address: call.contract_address.to_hex_string(),
            entrypoint: entrypoint.clone(),
            calldata: call.calldata.iter().map(Felt::to_hex_string).collect(),
        })
    }

    async fn run_cli<T: DeserializeOwned>(&self, args: &[&str]) -> Result<T> {
        self.run_cli_streaming(args, |_| {}).await
    }

    async fn run_cli_streaming<T, F>(&self, args: &[&str], mut on_event: F) -> Result<T>
    where
        T: DeserializeOwned,
        F: FnMut(CliEvent),
    {
        let mut args: Vec<&str> = args.to_vec();
        if !args.iter().any(|a| *a == "--json") {
            args.push("--json");
        }

        info!("{} {}", self.path.display(), args.join(" "));

        let mut child = Command::new(self.path.clone())
            .args(&args)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|_| {
                GameError::CartridgeCliError(CartridgeCliError::FailedToSpawnCli(self.path.clone()))
            })?;

        // Drain stderr concurrently so the CLI's diagnostics aren't lost.
        if let Some(stderr) = child.stderr.take() {
            tokio::spawn(async move {
                let mut err_lines = BufReader::new(stderr).lines();
                while let Ok(Some(line)) = err_lines.next_line().await {
                    if !line.trim().is_empty() {
                        error!("{line}");
                    }
                }
            });
        }

        let stdout = child.stdout.take().expect("stdout was piped");
        let mut lines = BufReader::new(stdout).lines();

        let mut last_success: Option<Value> = None;
        let mut pending_error: Option<String> = None;

        // The CLI emits pretty-printed JSON, so a single object spans many lines.
        // Accumulate into `buf` and parse once it forms a complete value; single-line
        // (NDJSON) output parses on the first line and clears the buffer immediately.
        let mut buf = String::new();

        while let Some(line) = lines.next_line().await.map_err(|e| {
            GameError::CartridgeCliError(CartridgeCliError::CliError(format!(
                "Failed reading controller output: {e}"
            )))
        })? {
            info!("{line}");

            buf.push_str(&line);
            buf.push('\n');

            let trimmed = buf.trim();
            if trimmed.is_empty() {
                buf.clear();
                continue;
            }

            let value = match serde_json::from_str::<Value>(trimmed) {
                Ok(value) => value,
                // Incomplete object spanning more lines — keep buffering.
                Err(e) if e.is_eof() => continue,
                // Genuinely malformed chunk — drop it and resync on the next line.
                Err(_) => {
                    buf.clear();
                    continue;
                }
            };
            buf.clear();

            let event = CliEvent::from_value(value);
            match &event {
                CliEvent::Success { data } => last_success = Some(data.clone()),
                CliEvent::Error { message, hint, .. } => {
                    pending_error = Some(CliEvent::error_string(message, hint.as_deref()));
                }
                _ => {}
            }

            on_event(event);

            if pending_error.is_some() {
                break;
            }
        }

        let _ = child.wait().await;

        if let Some(err) = pending_error {
            return Err(GameError::CartridgeCliError(CartridgeCliError::CliError(
                format!("{}", err),
            )));
        }

        let data = last_success.ok_or_else(|| {
            GameError::CartridgeCliError(CartridgeCliError::CliError(
                "No success event".to_string(),
            ))
        })?;

        serde_json::from_value(data).map_err(|e| {
            GameError::CartridgeCliError(CartridgeCliError::FailedToDeserialize(format!("{e}")))
        })
    }

    async fn temp_file(name: &str, content: String) -> Result<PathBuf> {
        let path =
            std::env::temp_dir().join(format!("starkwaves-{}-{}.json", name, Uuid::new_v4()));

        tokio::fs::write(&path, content).await.map_err(|e| {
            GameError::CartridgeCliError(CartridgeCliError::CliError(format!(
                "Failed to write calls file: {e}"
            )))
        })?;

        Ok(path)
    }

    async fn remove_temp_file(path: PathBuf) -> Result<()> {
        tokio::fs::remove_file(&path).await.map_err(|e| {
            GameError::CartridgeCliError(CartridgeCliError::CliError(format!(
                "Failed to delete calls file: {e}"
            )))
        })
    }

    fn policies_json(contract_address: Felt, policies: Vec<PolicyMethod>) -> String {
        json!({
            "contracts": {
                format!("{:#x}", contract_address): {
                    "name": "Starkwaves",
                    "methods": policies
                }
            }
        })
            .to_string()
    }
}
