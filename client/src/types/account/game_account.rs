use async_trait::async_trait;
use starknet_rust::accounts::{Account, ConnectedAccount, SingleOwnerAccount};
use crate::types::result::Result;
use starknet_rust::providers::JsonRpcClient;
use starknet_rust::providers::jsonrpc::HttpTransport;
use starknet_rust::signers::LocalWallet;
use starknet_rust_core::types::{Call, Felt, TransactionReceiptWithBlockInfo};
use crate::utils::wait_success;

#[async_trait]
pub trait GameAccount: Send + Sync {
    fn address(&self) -> Felt;

    async fn send(&self, calls: Vec<Call>) -> Result<Felt>;
    async fn send_and_wait(&self, calls: Vec<Call>) -> Result<TransactionReceiptWithBlockInfo>;
}
