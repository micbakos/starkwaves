use dotenv::dotenv;
use starknet_rust::accounts::{ExecutionEncoding, SingleOwnerAccount};
use starknet_rust::core::types::Felt;
use starknet_rust::core::utils::cairo_short_string_to_felt;
use starknet_rust::providers::jsonrpc::HttpTransport;
use starknet_rust::providers::JsonRpcClient;
use starknet_rust::signers::{LocalWallet, SigningKey};
use std::env;
use url::Url;

#[derive(Debug, Clone)]
pub struct PlayerPreset {
    pub private_key: Felt,
    pub address: Felt,
}

#[derive(Debug, Clone)]
pub struct Environment {
    rpc_url: Url,
    pub ws_url: Url,
    chain_id: Felt,
    pub contract_address: Felt,
    preset_a: Option<PlayerPreset>,
    preset_b: Option<PlayerPreset>,
}

fn read_player_preset(key_var: &str, addr_var: &str) -> Option<PlayerPreset> {
    let private_key = Felt::from_hex(&env::var(key_var).ok()?).ok()?;
    let address = Felt::from_hex(&env::var(addr_var).ok()?).ok()?;
    Some(PlayerPreset { private_key, address })
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

        Self {
            rpc_url: Url::parse(rpc_url_str.as_str()).expect("Invalid RPC_URL"),
            ws_url: Url::parse(ws_url_str.as_str()).expect("Invalid WS_URL"),
            chain_id: cairo_short_string_to_felt(chain_id_str.as_str()).expect("Invalid CHAIN_ID"),
            contract_address,
            preset_a: read_player_preset("PRESET_A_PRIVATE_KEY", "PRESET_A_ADDRESS"),
            preset_b: read_player_preset("PRESET_B_PRIVATE_KEY", "PRESET_B_ADDRESS"),
        }
    }
    
    pub fn rpc_provider(&self) -> JsonRpcClient<HttpTransport> {
        JsonRpcClient::new(HttpTransport::new(self.rpc_url.to_owned()))
    }

    pub fn player(
        &self,
        preset: Option<&str>,
        private_key: Option<&str>,
        address: Option<&str>,
        provider: &JsonRpcClient<HttpTransport>,
    ) -> SingleOwnerAccount<JsonRpcClient<HttpTransport>, LocalWallet> {
        let (private_key, address) = match preset {
            Some("A") | Some("a") => {
                let p = self.preset_a.as_ref().expect("PRESET_A_PRIVATE_KEY / PRESET_A_ADDRESS not set");
                (p.private_key, p.address)
            }
            Some("B") | Some("b") => {
                let p = self.preset_b.as_ref().expect("PRESET_B_PRIVATE_KEY / PRESET_B_ADDRESS not set");
                (p.private_key, p.address)
            }
            Some(other) => panic!("Unknown preset '{}'. Valid values: A, B", other),
            None => {
                let private_key = Felt::from_hex(private_key.expect("--private-key required when --preset is not set"))
                    .expect("Invalid private key format");
                let address = Felt::from_hex(address.expect("--address required when --preset is not set"))
                    .expect("Invalid address format");
                (private_key, address)
            }
        };

        let signer = LocalWallet::from(SigningKey::from_secret_scalar(private_key));

        SingleOwnerAccount::new(
            provider.clone(),
            signer,
            address,
            self.chain_id,
            ExecutionEncoding::New,
        )
    }
}
