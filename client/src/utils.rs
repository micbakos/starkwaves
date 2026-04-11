use starknet_rust::core::types::{Felt, StarknetError, TransactionReceiptWithBlockInfo};
use starknet_rust::providers::{Provider, ProviderError};
use std::time::Duration;
use tokio::time::sleep;

pub async fn wait_for_receipt<P: Provider>(
    provider: &P,
    tx_hash: Felt,
) -> Result<TransactionReceiptWithBlockInfo, ProviderError> {
    loop {
        match provider.get_transaction_receipt(tx_hash).await {
            Ok(receipt) => return Ok(receipt),
            Err(ProviderError::StarknetError(StarknetError::TransactionHashNotFound)) => {
                sleep(Duration::from_millis(500)).await;
            }
            Err(e) => return Err(e),
        }
    }
}
