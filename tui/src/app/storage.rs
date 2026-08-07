use crate::app::types::{AccountKind, LoggedAccount};
use crate::types::error::TuiError;
use crate::types::result::Result;
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use starknet_rust_core::types::Felt;
use std::fs::{File, create_dir, remove_file};
use std::io::{BufReader, ErrorKind, Write};

const DATA_FILE: &str = "data.json";

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct StoredSession {
    pub tui_version: String,
    pub contract_address: Felt,
    pub chain_id: Felt,
    pub account: StoredAccount,
}

impl StoredSession {
    pub fn new(contract_address: Felt, chain_id: Felt, account: StoredAccount) -> Self {
        Self {
            tui_version: env!("CARGO_PKG_VERSION").to_string(),
            contract_address,
            chain_id,
            account,
        }
    }

    pub fn read() -> Result<Option<StoredSession>> {
        let dir = Self::project_dir();
        let data_dir = dir.data_dir();

        let data_file = File::open(data_dir.join(DATA_FILE))
            .map(|f| Some(f))
            .or_else(|e| match e.kind() {
                ErrorKind::NotFound => Ok(None),
                _ => Err(TuiError::FailedToReadFromStorage(e.to_string())),
            })?;

        if let Some(data_file) = data_file {
            let reader = BufReader::new(data_file);
            serde_json::from_reader(reader)
                .map(Some)
                .map_err(|e| TuiError::FailedToReadFromStorage(e.to_string()))
        } else {
            Ok(None)
        }
    }

    pub fn store(&self) -> Result<()> {
        let dir = Self::project_dir();
        let data_dir = dir.data_dir();
        if !data_dir.exists() {
            create_dir(data_dir).map_err(|e| TuiError::FailedToWriteToStorage(e.to_string()))?;
        }

        let value = serde_json::to_string(self)
            .map_err(|e| TuiError::FailedToWriteToStorage(e.to_string()))?;
        let mut file = File::create(data_dir.join(DATA_FILE))
            .map_err(|e| TuiError::FailedToWriteToStorage(e.to_string()))?;
        file.write_all(&value.as_bytes())
            .map_err(|e| TuiError::FailedToWriteToStorage(e.to_string()))?;

        Ok(())
    }

    pub fn delete() -> Result<()> {
        let dir = Self::project_dir();
        let data_dir = dir.data_dir();
        remove_file(data_dir.join(DATA_FILE))
            .map_err(|e| TuiError::FailedToWriteToStorage(e.to_string()))
    }

    pub fn project_dir() -> ProjectDirs {
        ProjectDirs::from("com", "starkwaves", "tui").unwrap()
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct StoredAccount {
    pub address: Felt,
    pub username: String,
    pub kind: StoredAccountKind,
}

impl Into<LoggedAccount> for StoredAccount {
    fn into(self) -> LoggedAccount {
        LoggedAccount {
            address: self.address,
            username: self.username,
            kind: self.kind.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub enum StoredAccountKind {
    Cartridge,
    #[cfg(debug_assertions)]
    Env,
}

impl Into<AccountKind> for StoredAccountKind {
    fn into(self) -> AccountKind {
        match self {
            Self::Cartridge => AccountKind::Cartridge,
            #[cfg(debug_assertions)]
            StoredAccountKind::Env => AccountKind::Local,
        }
    }
}
