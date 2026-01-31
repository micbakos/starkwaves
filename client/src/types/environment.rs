use dotenv::dotenv;
use starknet::accounts::{ExecutionEncoding, SingleOwnerAccount};
use starknet::core::types::Felt;
use starknet::core::utils::cairo_short_string_to_felt;
use starknet::providers::jsonrpc::HttpTransport;
use starknet::providers::JsonRpcClient;
use starknet::signers::{LocalWallet, SigningKey};
use std::env;
use log::debug;
use url::Url;

#[derive(Debug)]
pub struct Environment {
    rpc_url: Url,
    pub ws_url: Url,
    chain_id: Felt,
    pub contract_address: Felt,
    player_a_address: Felt,
    player_a_key: Felt,
    player_b_address: Felt,
}

impl Environment {
    pub fn new() -> Self {
        dotenv().ok();
        let preset = env::var("PRESET").unwrap_or_else(|_| "Should have PRESET in .env".to_string());
        dotenv::from_filename(format!(".env.{}", preset)).ok();

        let chain_id_str = env::var("CHAIN_ID").expect("Should have CHAIN_ID in .env");
        let rpc_url_str = env::var("RPC_URL").expect("Should have RPC_URL in .env");
        let ws_url_str = env::var("WS_URL").expect("Should have WS_URL in .env");

        let contract_address_str =
            env::var("CONTRACT_ADDR").expect("Should have CONTRACT_ADDRESS in .env.\nRun: \n\tcargo run --bin deploy");
        let contract_address =
            Felt::from_hex(contract_address_str.as_str()).expect("Invalid contract address");
        let player_a_address =
            env::var("PLAYER_A_ADDR").expect("Should have PLAYER_A_ADDR in .env");
        let player_a_key = env::var("PLAYER_A_KEY").expect("Should have PLAYER_A_KEY in .env");

        let player_b_address =
            env::var("PLAYER_B_ADDR").expect("Should have PLAYER_B_ADDR in .env");
        
        Self {
            rpc_url: Url::parse(rpc_url_str.as_str()).expect("Invalid RPC_URL"),
            ws_url: Url::parse(ws_url_str.as_str()).expect("Invalid WS_URL"),
            chain_id: cairo_short_string_to_felt(chain_id_str.as_str()).expect("Invalid CHAIN_ID"),
            contract_address,
            player_a_address: Felt::from_hex(player_a_address.as_str()).expect("Invalid address"),
            player_a_key: Felt::from_hex(player_a_key.as_str()).expect("Invalid private key"),
            player_b_address: Felt::from_hex(player_b_address.as_str()).expect("Invalid address"),
        }
    }

    pub fn rpc_provider(&self) -> JsonRpcClient<HttpTransport> {
        JsonRpcClient::new(HttpTransport::new(self.rpc_url.to_owned()))
    }

    pub fn host(
        &self,
        provider: &JsonRpcClient<HttpTransport>,
    ) -> SingleOwnerAccount<JsonRpcClient<HttpTransport>, LocalWallet> {
        let private_key = self.player_a_key.clone();
        let signer = LocalWallet::from(SigningKey::from_secret_scalar(private_key));

        SingleOwnerAccount::new(
            provider.clone(),
            signer,
            self.player_a_address,
            self.chain_id,
            ExecutionEncoding::New,
        )
    }

    pub fn opponent(&self) -> Felt {
        self.player_b_address
    }
}
