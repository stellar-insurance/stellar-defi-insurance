use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Instant;
use crate::services::insurance_store::InsuranceStore;

#[derive(Clone, Debug)]
pub struct Config {
    pub bind_addr: SocketAddr,
    pub contract_id: String,
    pub soroban_rpc_url: String,
    pub horizon_url: String,
    pub network_passphrase: String,
    pub network_name: String,
}

impl Config {
    pub fn from_env() -> Self {
        let port: u16 = std::env::var("PORT").ok().and_then(|p| p.parse().ok()).unwrap_or(3000);
        Self {
            bind_addr: format!("0.0.0.0:{port}").parse().unwrap_or_else(|_| "0.0.0.0:3000".parse().unwrap()),
            contract_id: std::env::var("CONTRACT_ID").unwrap_or_default(),
            soroban_rpc_url: std::env::var("SOROBAN_RPC_URL").unwrap_or_else(|_| "https://soroban-testnet.stellar.org:443".into()),
            horizon_url: std::env::var("HORIZON_URL").unwrap_or_else(|_| "https://horizon-testnet.stellar.org".into()),
            network_passphrase: std::env::var("NETWORK_PASSPHRASE").unwrap_or_else(|_| "Test SDF Network ; September 2015".into()),
            network_name: std::env::var("NETWORK_NAME").unwrap_or_else(|_| "TESTNET".into()),
        }
    }
}

#[derive(Clone)]
pub struct AppState {
    pub config: Config,
    pub store: Arc<InsuranceStore>,
    pub started_at: Instant,
}

impl AppState {
    pub fn new(config: Config) -> Self {
        Self {
            config,
            store: Arc::new(InsuranceStore::new()),
            started_at: Instant::now(),
        }
    }
}
