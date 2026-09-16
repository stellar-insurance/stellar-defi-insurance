use std::collections::HashMap;
use std::sync::RwLock;
use crate::error::ApiError;
use crate::models::{PolicyDto, PolicyStatus, ClaimDto, ClaimStatus};

const DEFAULT_PREMIUM_BPS: i128 = 500;
const MIN_COVERAGE: i128 = 1_000;
const MAX_COVERAGE: i128 = 1_000_000_000;
const MAX_DURATION: u64 = 90 * 24 * 60 * 60;

#[derive(Debug, Clone)]
struct StoredPolicy {
    id: u64, holder: String, covered_contract: String,
    coverage_amount: i128, premium_paid: i128,
    start_time: u64, expiry_time: u64,
    status: PolicyStatus, ledger: u32,
}

#[derive(Debug, Clone)]
struct StoredClaim {
    id: u64, policy_id: u64, claimant: String,
    claim_amount: i128, evidence: String,
    status: ClaimStatus, assessor: Option<String>,
    filed_at: u64, reviewed_at: Option<u64>,
}

pub struct InsuranceStore {
    policies: RwLock<Vec<StoredPolicy>>,
    claims: RwLock<Vec<StoredClaim>>,
    assessors: RwLock<HashMap<String, bool>>,
    admin: RwLock<Option<String>>,
    pool: RwLock<i128>,
    premium_bps: RwLock<i128>,
    paused: RwLock<bool>,
    now: RwLock<u64>,
}

impl InsuranceStore {
    pub fn new() -> Self {
        let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();
        Self {
            policies: RwLock::new(Vec::new()),
            claims: RwLock::new(Vec::new()),
            assessors: RwLock::new(HashMap::new()),
            admin: RwLock::new(None),
            pool: RwLock::new(0),
            premium_bps: RwLock::new(DEFAULT_PREMIUM_BPS),
            paused: RwLock::new(false),
            now: RwLock::new(now),
        }
    }

    pub fn initialize(&self, admin: String) -> Result<(), ApiError> {
        let mut cur = self.admin.write().unwrap();
        if cur.is_some() { return Err(ApiError::InvalidInput("Already initialized".into())); }
        *cur = Some(admin); Ok(())
    }

    pub fn is_initialized(&self) -> bool { self.admin.read().unwrap().is_some() }
    pub fn get_admin(&self) -> Result<String, ApiError> { self.admin.read().unwrap().clone().ok_or(ApiError::NotAuthorized) }
    pub fn get_pool(&self) -> i128 { *self.pool.read().unwrap() }
    pub fn get_premium_bps(&self) -> i128 { *self.premium_bps.read().unwrap() }
    pub fn set_premium_bps(&self, bps: i128) { *self.premium_bps.write().unwrap() = bps; }
    pub fn policy_count(&self) -> u64 { self.policies.read().unwrap().len() as u64 }
    pub fn claim_count(&self) -> u64 { self.claims.read().unwrap().len() as u64 }
    pub fn is_paused(&self) -> bool { *self.paused.read().unwrap() }
    pub fn pause(&self) { *self.paused.write().unwrap() = true; }
    pub fn unpause(&self) { *self.paused.write().unwrap() = false; }

    pub fn grant_assessor(&self, addr: &str) { self.assessors.write().unwrap().insert(addr.into(), true); }
    pub fn revoke_assessor(&self, addr: &str) { self.assessors.write().unwrap().remove(addr); }
    pub fn is_assessor(&self, addr: &str) -> bool { *self.assessors.read().unwrap().get(addr).unwrap_or(&false) }

    pub fn purchase_policy(&self, holder: &str, covered: &str, coverage: i128, duration: u64) -> Result<u64, ApiError> {
        if *self.paused.read().unwrap() { return Err(ApiError::InvalidInput("Protocol paused".into())); }
        if coverage < MIN_COVERAGE || coverage > MAX_COVERAGE { return Err(ApiError::InvalidAmount(format!("Coverage must be {MIN_COVERAGE}..{MAX_COVERAGE}"))); }
        if duration == 0 || duration > MAX_DURATION { return Err(ApiError::InvalidInput("Invalid duration".into())); }
        if holder == covered { return Err(ApiError::InvalidInput("Cannot insure self".into())); }
        let now = *self.now.read().unwrap();
        let premium = (coverage * *self.premium_bps.read().unwrap()) / 10_000;
        let mut policies = self.policies.write().unwrap();
        let id = policies.len() as u64;
        policies.push(StoredPolicy {
            id, holder: holder.into(), covered_contract: covered.into(),
            coverage_amount: coverage, premium_paid: premium,
            start_time: now, expiry_time: now + duration,
            status: PolicyStatus::Active, ledger: 1,
        });
        *self.pool.write().unwrap() += premium;
        Ok(id)
    }

    pub fn get_policy(&self, id: u64) -> Result<PolicyDto, ApiError> {
        self.policies.read().unwrap().get(id as usize)
            .map(|p| p.to_dto()).ok_or(ApiError::PolicyNotFound(id))
    }

    pub fn get_all_policies(&self) -> Vec<PolicyDto> {
        self.policies.read().unwrap().iter().map(|p| p.to_dto()).collect()
    }

    pub fn file_claim(&self, claimant: &str, policy_id: u64, amount: i128, evidence: &str) -> Result<u64, ApiError> {
        let mut policies = self.policies.write().unwrap();
        let policy = policies.get(policy_id as usize).ok_or(ApiError::PolicyNotFound(policy_id))?;
        if policy.holder != claimant { return Err(ApiError::NotAuthorized); }
        if policy.status != PolicyStatus::Active { return Err(ApiError::PolicyNotActive); }
        let now = *self.now.read().unwrap();
        if now >= policy.expiry_time { return Err(ApiError::PolicyExpired); }
        if amount <= 0 || amount > policy.coverage_amount { return Err(ApiError::InvalidAmount("Claim amount exceeds coverage".into())); }
        if evidence.is_empty() { return Err(ApiError::InvalidInput("Evidence required".into())); }

        // Mark policy as claimed
        policies[policy_id as usize].status = PolicyStatus::Claimed;
        drop(policies);

        let mut claims = self.claims.write().unwrap();
        let id = claims.len() as u64;
        claims.push(StoredClaim {
            id, policy_id, claimant: claimant.into(), claim_amount: amount,
            evidence: evidence.into(), status: ClaimStatus::Pending,
            assessor: None, filed_at: now, reviewed_at: None,
        });
        Ok(id)
    }

    pub fn get_claim(&self, id: u64) -> Result<ClaimDto, ApiError> {
        self.claims.read().unwrap().get(id as usize)
            .map(|c| c.to_dto()).ok_or(ApiError::ClaimNotFound(id))
    }

    pub fn get_all_claims(&self) -> Vec<ClaimDto> {
        self.claims.read().unwrap().iter().map(|c| c.to_dto()).collect()
    }

    pub fn approve_claim(&self, claim_id: u64, assessor: &str) -> Result<(), ApiError> {
        if !self.is_assessor(assessor) { return Err(ApiError::NotAuthorized); }
        let mut claims = self.claims.write().unwrap();
        let claim = claims.get_mut(claim_id as usize).ok_or(ApiError::ClaimNotFound(claim_id))?;
        if claim.status != ClaimStatus::Pending { return Err(ApiError::ClaimAlreadyProcessed); }
        let pool = *self.pool.read().unwrap();
        if pool < claim.claim_amount { return Err(ApiError::InsufficientPool); }
        let now = *self.now.read().unwrap();
        claim.status = ClaimStatus::Approved;
        claim.assessor = Some(assessor.into());
        claim.reviewed_at = Some(now);
        Ok(())
    }

    pub fn reject_claim(&self, claim_id: u64, assessor: &str) -> Result<(), ApiError> {
        if !self.is_assessor(assessor) { return Err(ApiError::NotAuthorized); }
        let mut claims = self.claims.write().unwrap();
        let claim = claims.get_mut(claim_id as usize).ok_or(ApiError::ClaimNotFound(claim_id))?;
        if claim.status != ClaimStatus::Pending { return Err(ApiError::ClaimAlreadyProcessed); }
        let now = *self.now.read().unwrap();
        claim.status = ClaimStatus::Rejected;
        claim.assessor = Some(assessor.into());
        claim.reviewed_at = Some(now);
        // Restore policy
        let mut policies = self.policies.write().unwrap();
        if let Some(p) = policies.get_mut(claim.policy_id as usize) {
            if p.status == PolicyStatus::Claimed { p.status = PolicyStatus::Active; }
        }
        Ok(())
    }

    pub fn mark_paid(&self, claim_id: u64) -> Result<(), ApiError> {
        let mut claims = self.claims.write().unwrap();
        let claim = claims.get_mut(claim_id as usize).ok_or(ApiError::ClaimNotFound(claim_id))?;
        if claim.status != ClaimStatus::Approved { return Err(ApiError::ClaimAlreadyProcessed); }
        *self.pool.write().unwrap() -= claim.claim_amount;
        claim.status = ClaimStatus::Paid;
        Ok(())
    }

    pub fn withdraw_pool(&self, amount: i128) -> Result<(), ApiError> {
        if amount <= 0 { return Err(ApiError::InvalidAmount("Must be positive".into())); }
        let mut pool = self.pool.write().unwrap();
        if *pool < amount { return Err(ApiError::InsufficientPool); }
        *pool -= amount;
        Ok(())
    }

    pub fn seed_demo_data(&self) {
        if self.is_initialized() { return; }
        let _ = self.initialize("GDQP2PQPM3XKKD4FQ7LG7N4XQ4FQ7LG7N4XQ4FQ7LG7N4XQ4FQ7".into());
        let _admin = "GDQP2PQPM3XKKD4FQ7LG7N4XQ4FQ7LG7N4XQ4FQ7LG7N4XQ4FQ7";
        let assessor = "GA2C5RFPE6GCKMY3US5GAB29QZYE7DYXTZPGHLDJ5YBJO7VOZKAUBP2X";
        self.grant_assessor(assessor);

        let holder1 = "GDFX4P3OZO5NHNXHKDI5G4QXTZKQO4NHPKWXDWQZQFILXKZKABZ6WZ2N";
        let holder2 = "GCXV2VWQFJZWCY2PHEJF5XMQ5RZQ3PKXEXZQKOOYZOJXFFAAYQJYTSK";
        let covered1 = "GBXGIGJZGLESPGKZQ3K6X6DHPKQK5Y5Q5F7P4M2D6J5L5XK5Y5Q5F7P";
        let covered2 = "GCTZ5Q7K6X6DHPKQK5Y5Q5F7P4M2D6J5L5XK5Y5Q5F7P4M2D6J5L5XK5";

        let _ = self.purchase_policy(holder1, covered1, 50_000, 86400);
        let _ = self.purchase_policy(holder2, covered2, 100_000, 172800);
        let _ = self.purchase_policy(holder1, covered2, 25_000, 3600);
        let _ = self.purchase_policy(holder2, covered1, 75_000, 604800);

        // File a claim on policy 0
        let _ = self.file_claim(holder1, 0, 30_000, "Reentrancy exploit detected in covered contract. TX: 0xabc123");
        // Approve it (assessor)
        let _ = self.approve_claim(0, assessor);

        tracing::info!("Seeded 4 policies, 1 claim (approved)");
    }
}

impl StoredPolicy {
    fn to_dto(&self) -> PolicyDto {
        PolicyDto {
            id: self.id, holder: self.holder.clone(), covered_contract: self.covered_contract.clone(),
            coverage_amount: self.coverage_amount, premium_paid: self.premium_paid,
            start_time: self.start_time, expiry_time: self.expiry_time,
            status: self.status, ledger: self.ledger,
        }
    }
}

impl StoredClaim {
    fn to_dto(&self) -> ClaimDto {
        ClaimDto {
            id: self.id, policy_id: self.policy_id, claimant: self.claimant.clone(),
            claim_amount: self.claim_amount, evidence: self.evidence.clone(),
            status: self.status, assessor: self.assessor.clone(),
            filed_at: self.filed_at, reviewed_at: self.reviewed_at,
        }
    }
}

impl Default for InsuranceStore { fn default() -> Self { Self::new() } }
