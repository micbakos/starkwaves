use crate::types::result::Result;
use async_trait::async_trait;
use starknet_rust_core::types::{Call, Felt, TransactionReceiptWithBlockInfo};

#[async_trait]
pub trait GameAccount: Send + Sync {
    fn address(&self) -> Felt;

    async fn send(&self, calls: Vec<Call>) -> Result<Felt>;
    async fn send_and_wait(&self, calls: Vec<Call>) -> Result<TransactionReceiptWithBlockInfo>;
}
