use async_trait::async_trait;
use starknet_rust::accounts::{Account, ConnectedAccount, SingleOwnerAccount};
use starknet_rust::providers::jsonrpc::HttpTransport;
use starknet_rust::providers::JsonRpcClient;
use starknet_rust::signers::LocalWallet;
use starknet_rust_core::types::{Call, Felt, TransactionReceiptWithBlockInfo};
use crate::types::account::game_account::GameAccount;
use crate::utils::wait_success;

pub type LocalAccount = SingleOwnerAccount<JsonRpcClient<HttpTransport>, LocalWallet>; 

#[async_trait]
impl GameAccount for LocalAccount {
    fn address(&self) -> Felt {
        Account::address(&self)
    }

    async fn send(&self, calls: Vec<Call>) -> crate::types::result::Result<Felt> {
        let result = self
            .execute_v3(calls)
            .gas_estimate_multiplier(5.0)
            .send()
            .await
            .map_err(|e| e.into())?;

        Ok(result.transaction_hash)
    }

    async fn send_and_wait(&self, calls: Vec<Call>) -> crate::types::result::Result<TransactionReceiptWithBlockInfo> {
        let tx_hash = self.send(calls).await?;

        wait_success(self.provider(), tx_hash)
            .await
            .map_err(|e| e.into())
    }
}