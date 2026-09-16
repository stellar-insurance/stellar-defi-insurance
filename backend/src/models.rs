use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PolicyStatus { Active, Expired, Claimed, Cancelled }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ClaimStatus { Pending, Approved, Rejected, Paid }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyDto {
    pub id: u64,
    pub holder: String,
    pub covered_contract: String,
    pub coverage_amount: i128,
    pub premium_paid: i128,
    pub start_time: u64,
    pub expiry_time: u64,
    pub status: PolicyStatus,
    pub ledger: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaimDto {
    pub id: u64,
    pub policy_id: u64,
    pub claimant: String,
    pub claim_amount: i128,
    pub evidence: String,
    pub status: ClaimStatus,
    pub assessor: Option<String>,
    pub filed_at: u64,
    pub reviewed_at: Option<u64>,
}

#[derive(Debug, Deserialize)]
pub struct PurchasePolicyRequest {
    pub holder: String,
    pub covered_contract: String,
    pub coverage_amount: i128,
    pub duration_seconds: u64,
}

#[derive(Debug, Deserialize)]
pub struct FileClaimRequest {
    pub claimant: String,
    pub policy_id: u64,
    pub claim_amount: i128,
    pub evidence: String,
}

#[derive(Debug, Deserialize)]
pub struct ReviewClaimRequest {
    pub assessor: String,
    pub claim_id: u64,
}

#[derive(Debug, Serialize)]
pub struct HealthResponse { pub status: &'static str, pub version: &'static str, pub uptime_seconds: u64 }

#[derive(Debug, Serialize)]
pub struct StatusResponse {
    pub status: &'static str, pub version: &'static str, pub uptime_seconds: u64,
    pub network: String, pub contract_id: String, pub policy_count: u64, pub claim_count: u64,
    pub pool_balance: i128, pub premium_rate_bps: i128,
}

#[derive(Debug, Serialize)]
pub struct OperationResult { pub success: bool, pub message: String }

#[derive(Debug, Serialize)]
pub struct PoolResponse { pub pool_balance: i128, pub premium_rate_bps: i128 }

#[derive(Debug, Serialize)]
pub struct AssessorResponse { pub address: String, pub is_assessor: bool }

#[derive(Debug, Deserialize)]
pub struct AssessorRoleRequest { pub admin: String, pub address: String }
