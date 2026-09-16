use axum::extract::State;
use axum::Json;
use crate::models::{HealthResponse, StatusResponse};
use crate::state::AppState;

pub async fn health(State(state): State<AppState>) -> Json<HealthResponse> {
    Json(HealthResponse { status: "ok", version: env!("CARGO_PKG_VERSION"), uptime_seconds: state.started_at.elapsed().as_secs() })
}

pub async fn status(State(state): State<AppState>) -> Json<StatusResponse> {
    let c = &state.config;
    Json(StatusResponse {
        status: "ok", version: env!("CARGO_PKG_VERSION"), uptime_seconds: state.started_at.elapsed().as_secs(),
        network: c.network_name.clone(), contract_id: c.contract_id.clone(),
        policy_count: state.store.policy_count(), claim_count: state.store.claim_count(),
        pool_balance: state.store.get_pool(), premium_rate_bps: state.store.get_premium_bps(),
    })
}
