use axum::extract::{Path, State};
use axum::Json;
use crate::error::ApiError;
use crate::models::*;
use crate::state::AppState;

pub async fn list_policies(State(s): State<AppState>) -> Json<Vec<PolicyDto>> {
    Json(s.store.get_all_policies())
}

pub async fn get_policy(State(s): State<AppState>, Path(id): Path<u64>) -> Result<Json<PolicyDto>, ApiError> {
    Ok(Json(s.store.get_policy(id)?))
}

pub async fn policy_count(State(s): State<AppState>) -> Json<serde_json::Value> {
    Json(serde_json::json!({ "count": s.store.policy_count() }))
}

pub async fn purchase_policy(State(s): State<AppState>, Json(req): Json<PurchasePolicyRequest>) -> Result<Json<OperationResult>, ApiError> {
    let id = s.store.purchase_policy(&req.holder, &req.covered_contract, req.coverage_amount, req.duration_seconds)?;
    tracing::info!(policy_id = id, "Policy purchased");
    Ok(Json(OperationResult { success: true, message: format!("Policy {id} purchased") }))
}

pub async fn list_claims(State(s): State<AppState>) -> Json<Vec<ClaimDto>> {
    Json(s.store.get_all_claims())
}

pub async fn get_claim(State(s): State<AppState>, Path(id): Path<u64>) -> Result<Json<ClaimDto>, ApiError> {
    Ok(Json(s.store.get_claim(id)?))
}

pub async fn claim_count(State(s): State<AppState>) -> Json<serde_json::Value> {
    Json(serde_json::json!({ "count": s.store.claim_count() }))
}

pub async fn file_claim(State(s): State<AppState>, Json(req): Json<FileClaimRequest>) -> Result<Json<OperationResult>, ApiError> {
    let id = s.store.file_claim(&req.claimant, req.policy_id, req.claim_amount, &req.evidence)?;
    tracing::info!(claim_id = id, "Claim filed");
    Ok(Json(OperationResult { success: true, message: format!("Claim {id} filed") }))
}

pub async fn approve_claim(State(s): State<AppState>, Json(req): Json<ReviewClaimRequest>) -> Result<Json<OperationResult>, ApiError> {
    s.store.approve_claim(req.claim_id, &req.assessor)?;
    Ok(Json(OperationResult { success: true, message: format!("Claim {} approved", req.claim_id) }))
}

pub async fn reject_claim(State(s): State<AppState>, Json(req): Json<ReviewClaimRequest>) -> Result<Json<OperationResult>, ApiError> {
    s.store.reject_claim(req.claim_id, &req.assessor)?;
    Ok(Json(OperationResult { success: true, message: format!("Claim {} rejected", req.claim_id) }))
}

pub async fn mark_paid(State(s): State<AppState>, Path(id): Path<u64>) -> Result<Json<OperationResult>, ApiError> {
    s.store.mark_paid(id)?;
    Ok(Json(OperationResult { success: true, message: format!("Claim {id} paid") }))
}

pub async fn pool_info(State(s): State<AppState>) -> Json<PoolResponse> {
    Json(PoolResponse { pool_balance: s.store.get_pool(), premium_rate_bps: s.store.get_premium_bps() })
}

pub async fn withdraw_pool(State(s): State<AppState>, Json(req): Json<serde_json::Value>) -> Result<Json<OperationResult>, ApiError> {
    let amount = req.get("amount").and_then(|v| v.as_i64()).unwrap_or(0) as i128;
    s.store.withdraw_pool(amount)?;
    Ok(Json(OperationResult { success: true, message: format!("Withdrew {amount} from pool") }))
}

pub async fn get_admin(State(s): State<AppState>) -> Result<Json<serde_json::Value>, ApiError> {
    let admin = s.store.get_admin()?;
    Ok(Json(serde_json::json!({ "admin": admin })))
}

pub async fn check_assessor(State(s): State<AppState>, Path(addr): Path<String>) -> Json<AssessorResponse> {
    Json(AssessorResponse { address: addr.clone(), is_assessor: s.store.is_assessor(&addr) })
}

pub async fn grant_assessor(State(_s): State<AppState>, Json(req): Json<AssessorRoleRequest>) -> Json<OperationResult> {
    _s.store.grant_assessor(&req.address);
    Json(OperationResult { success: true, message: format!("Assessor role granted to {}", req.address) })
}

pub async fn revoke_assessor(State(_s): State<AppState>, Json(req): Json<AssessorRoleRequest>) -> Json<OperationResult> {
    _s.store.revoke_assessor(&req.address);
    Json(OperationResult { success: true, message: format!("Assessor role revoked from {}", req.address) })
}

pub async fn pause(State(s): State<AppState>) -> Json<OperationResult> {
    s.store.pause(); Json(OperationResult { success: true, message: "Protocol paused".into() })
}

pub async fn unpause(State(s): State<AppState>) -> Json<OperationResult> {
    s.store.unpause(); Json(OperationResult { success: true, message: "Protocol resumed".into() })
}

pub async fn is_paused(State(s): State<AppState>) -> Json<serde_json::Value> {
    Json(serde_json::json!({ "paused": s.store.is_paused() }))
}
