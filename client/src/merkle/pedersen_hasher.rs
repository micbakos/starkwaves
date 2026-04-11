use starknet_rust::core::crypto::pedersen_hash;
use starknet_rust::core::types::Felt;

#[derive(Debug, Clone)]
pub struct PedersenHasher;

impl rs_merkle::Hasher for PedersenHasher {
    type Hash = [u8; 32];

    fn hash(data: &[u8]) -> Self::Hash {
        let mid = data.len() / 2;
        let left = Felt::from_bytes_be_slice(&data[..mid]);
        let right = Felt::from_bytes_be_slice(&data[mid..]);

        let hash = pedersen_hash(&left, &right);
        hash.to_bytes_be()
    }
}