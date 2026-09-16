pub mod health;
pub mod insurance;

use axum::routing::{get, post};
use axum::Router;
use crate::state::AppState;

pub fn build_router(state: AppState) -> Router {
    let api = Router::new()
        .route("/status", get(health::status))
        .route("/policies", get(insurance::list_policies).post(insurance::purchase_policy))
        .route("/policies/count", get(insurance::policy_count))
        .route("/policies/:id", get(insurance::get_policy))
        .route("/claims", get(insurance::list_claims).post(insurance::file_claim))
        .route("/claims/count", get(insurance::claim_count))
        .route("/claims/:id", get(insurance::get_claim))
        .route("/claims/approve", post(insurance::approve_claim))
        .route("/claims/reject", post(insurance::reject_claim))
        .route("/claims/:id/pay", post(insurance::mark_paid))
        .route("/pool", get(insurance::pool_info))
        .route("/pool/withdraw", post(insurance::withdraw_pool))
        .route("/admin", get(insurance::get_admin))
        .route("/assessors/:addr", get(insurance::check_assessor))
        .route("/assessors/grant", post(insurance::grant_assessor))
        .route("/assessors/revoke", post(insurance::revoke_assessor))
        .route("/pause", post(insurance::pause))
        .route("/unpause", post(insurance::unpause))
        .route("/paused", get(insurance::is_paused));

    Router::new()
        .route("/health", get(health::health))
        .nest("/api/v1", api)
        .with_state(state)
}
