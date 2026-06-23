use crate::types::error::GameError;
use crate::types::result::Result;
use starknet_rust::core::types::{
    ExecutionResult, Felt, StarknetError, TransactionReceiptWithBlockInfo,
};
use starknet_rust::providers::{Provider, ProviderError};
use std::time::Duration;
use tokio::time::sleep;

const WAIT_SUCCESS_MAX_ATTEMPTS: u32 = 60;
const WAIT_SUCCESS_POLL_INTERVAL: Duration = Duration::from_millis(500);

pub async fn wait_success<P: Provider>(
    provider: &P,
    tx_hash: Felt,
) -> Result<TransactionReceiptWithBlockInfo> {
    for _ in 0..WAIT_SUCCESS_MAX_ATTEMPTS {
        match provider.get_transaction_receipt(tx_hash).await {
            Ok(receipt) => {
                if let ExecutionResult::Reverted { reason } = receipt.receipt.execution_result() {
                    return Err(GameError::TxReverted {
                        tx_hash: tx_hash.to_fixed_hex_string(),
                        reason: reason.to_string(),
                    });
                }
                return Ok(receipt);
            }
            Err(ProviderError::StarknetError(StarknetError::TransactionHashNotFound)) => {
                sleep(WAIT_SUCCESS_POLL_INTERVAL).await;
            }
            Err(e) => return Err(e.into()),
        }
    }

    Err(ProviderError::StarknetError(StarknetError::TransactionHashNotFound).into())
}
