use async_trait::async_trait;
use starknet_rust::accounts::{Account, ConnectedAccount, SingleOwnerAccount};
use crate::types::result::Result;
use starknet_rust::providers::JsonRpcClient;
use starknet_rust::providers::jsonrpc::HttpTransport;
use starknet_rust::signers::LocalWallet;
use starknet_rust_core::types::{Call, Felt, TransactionReceiptWithBlockInfo};
use crate::utils::wait_success;

#[async_trait]
pub trait GameAccount: ConnectedAccount + Sync {
    async fn send(&self, calls: Vec<Call>) -> Result<Felt>;
    async fn send_and_wait(&self, calls: Vec<Call>) -> Result<TransactionReceiptWithBlockInfo>;
}

#[async_trait]
impl GameAccount for SingleOwnerAccount<JsonRpcClient<HttpTransport>, LocalWallet> {
    async fn send(&self, calls: Vec<Call>) -> Result<Felt> {
        let result = self
            .execute_v3(calls)
            .gas_estimate_multiplier(5.0)
            .send()
            .await
            .map_err(|e| e.into())?;

        Ok(result.transaction_hash)
    }

    async fn send_and_wait(&self, calls: Vec<Call>) -> Result<TransactionReceiptWithBlockInfo> {
        let tx_hash = self.send(calls).await?;

        wait_success(self.provider(), tx_hash)
            .await
            .map_err(|e| e.into())
    }
}
