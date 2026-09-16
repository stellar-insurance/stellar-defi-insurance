#![no_std]
//! # Stellar DeFi Insurance — Soroban Contract
//!
//! A **decentralized insurance protocol** on the Stellar network that provides
//! coverage for smart contract risks. Users purchase policies by paying
//! premiums into a coverage pool. If a covered event (e.g., a smart contract
//! exploit) occurs, policyholders can file claims that are validated by
//! designated claim assessors.
//!
//! ## Roles
//! - **Admin** — set once at [`initialize`]. Manages assessors, sets premium
//!   rates, and can pause/unpause the protocol.
//! - **Assessor** — approved by admin. Reviews and approves/rejects claims.
//! - **Policyholder** — any address that purchases a policy.
//!
//! ## Core flows
//! 1. **Purchase:** User pays a premium → receives a policy with coverage
//!    amount and expiry.
//! 2. **Claim:** Policyholder files a claim with evidence → assessor reviews →
//!    approves or rejects.
//! 3. **Payout:** On approval, the coverage amount is marked as paid.
//! 4. **Pool:** All premiums accumulate in the coverage pool. The admin can
//!    withdraw pool funds for payouts.

use soroban_sdk::{
    contract, contracterror, contractimpl, contractmeta, contracttype, panic_with_error,
    symbol_short, Address, Env, Map, String, Vec,
};

contractmeta!(
    key = "Description",
    val = "DeFi insurance protocol for smart contract risk coverage on Stellar"
);

/// Maximum policy duration in seconds (90 days).
const MAX_POLICY_DURATION: u64 = 90 * 24 * 60 * 60;
/// Minimum coverage amount (1 unit = 1 stroop equivalent).
const MIN_COVERAGE: i128 = 1_000;
/// Maximum coverage per policy.
const MAX_COVERAGE: i128 = 1_000_000_000;
/// Basis points for premium calculation (10000 = 100%).
const PREMIUM_BPS_DENOM: i128 = 10_000;
/// Default premium rate in basis points (5%).
const DEFAULT_PREMIUM_BPS: i128 = 500;
/// Maximum claim evidence length.
const MAX_EVIDENCE_LEN: u32 = 1024;

/// Status of a policy.
#[contracttype]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PolicyStatus {
    Active = 0,
    Expired = 1,
    Claimed = 2,
    Cancelled = 3,
}

/// Status of a claim.
#[contracttype]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ClaimStatus {
    Pending = 0,
    Approved = 1,
    Rejected = 2,
    Paid = 3,
}

/// An insurance policy.
#[contracttype]
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Policy {
    pub id: u64,
    pub holder: Address,
    /// Contract address being insured.
    pub covered_contract: Address,
    /// Coverage amount in base units.
    pub coverage_amount: i128,
    /// Premium paid in base units.
    pub premium_paid: i128,
    /// Ledger timestamp when policy starts.
    pub start_time: u64,
    /// Ledger timestamp when policy expires.
    pub expiry_time: u64,
    pub status: PolicyStatus,
    /// Ledger sequence when created.
    pub ledger: u32,
}

/// A claim filed against a policy.
#[contracttype]
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Claim {
    pub id: u64,
    pub policy_id: u64,
    pub claimant: Address,
    /// Amount being claimed.
    pub claim_amount: i128,
    /// Evidence or description of the exploit/loss.
    pub evidence: String,
    pub status: ClaimStatus,
    /// Assessor who reviewed (if any).
    pub assessor: Option<Address>,
    pub filed_at: u64,
    pub reviewed_at: Option<u64>,
}

/// Storage keys.
#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,
    Paused,
    PolicyCount,
    ClaimCount,
    PremiumBps,
    CoveragePool,
    Assessor(Address),
    Policy(u64),
    Claim(u64),
    HolderPolicies(Address),
}

/// Contract error codes.
#[contracterror]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u32)]
pub enum Error {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    NotAuthorized = 3,
    Paused = 4,
    InvalidAmount = 5,
    InvalidDuration = 6,
    PolicyNotFound = 7,
    ClaimNotFound = 8,
    PolicyExpired = 9,
    PolicyNotActive = 10,
    ClaimAlreadyFiled = 11,
    NotPolicyHolder = 12,
    ClaimAlreadyProcessed = 13,
    InsufficientPool = 14,
    InvalidEvidence = 15,
    NotAssessor = 16,
    SelfAssessment = 17,
}

#[contract]
pub struct InsuranceContract;

#[contractimpl]
impl InsuranceContract {
    /// One-time initialization. Sets the admin.
    pub fn initialize(env: Env, admin: Address) {
        if env.storage().instance().has(&DataKey::Admin) {
            panic_with_error!(&env, Error::AlreadyInitialized);
        }
        admin.require_auth();
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::Paused, &false);
        env.storage().instance().set(&DataKey::PolicyCount, &0u64);
        env.storage().instance().set(&DataKey::ClaimCount, &0u64);
        env.storage().instance().set(&DataKey::PremiumBps, &DEFAULT_PREMIUM_BPS);
        env.storage().instance().set(&DataKey::CoveragePool, &0i128);
        env.events().publish((symbol_short!("init"),), admin);
    }

    /// Get the admin address.
    pub fn get_admin(env: Env) -> Address {
        Self::read_admin(&env)
    }

    /// Set the premium rate in basis points. Admin-only.
    pub fn set_premium_rate(env: Env, bps: i128) {
        let admin = Self::read_admin(&env);
        admin.require_auth();
        if bps < 0 || bps > 10_000 {
            panic_with_error!(&env, Error::InvalidAmount);
        }
        env.storage().instance().set(&DataKey::PremiumBps, &bps);
        env.events().publish((symbol_short!("rate"), admin), bps);
    }

    /// Get current premium rate in basis points.
    pub fn get_premium_rate(env: Env) -> i128 {
        env.storage().instance().get(&DataKey::PremiumBps).unwrap_or(DEFAULT_PREMIUM_BPS)
    }

    /// Grant assessor role. Admin-only.
    pub fn grant_assessor(env: Env, who: Address) {
        let admin = Self::read_admin(&env);
        admin.require_auth();
        env.storage().persistent().set(&DataKey::Assessor(who.clone()), &true);
        env.events().publish((symbol_short!("granted"), admin), who);
    }

    /// Revoke assessor role. Admin-only.
    pub fn revoke_assessor(env: Env, who: Address) {
        let admin = Self::read_admin(&env);
        admin.require_auth();
        env.storage().persistent().remove(&DataKey::Assessor(who.clone()));
        env.events().publish((symbol_short!("revoked"), admin), who);
    }

    /// Check if address is an assessor.
    pub fn is_assessor(env: Env, who: Address) -> bool {
        env.storage().persistent().get(&DataKey::Assessor(who)).unwrap_or(false)
    }

    /// Pause the protocol. Admin-only.
    pub fn pause(env: Env) {
        let admin = Self::read_admin(&env);
        admin.require_auth();
        env.storage().instance().set(&DataKey::Paused, &true);
        env.events().publish((symbol_short!("paused"),), admin);
    }

    /// Unpause the protocol. Admin-only.
    pub fn unpause(env: Env) {
        let admin = Self::read_admin(&env);
        admin.require_auth();
        env.storage().instance().set(&DataKey::Paused, &false);
        env.events().publish((symbol_short!("unpaused"),), admin);
    }

    /// Is the protocol paused?
    pub fn is_paused(env: Env) -> bool {
        env.storage().instance().get(&DataKey::Paused).unwrap_or(false)
    }

    /// Purchase an insurance policy. Returns the policy ID.
    ///
    /// The caller pays a premium (calculated from the coverage amount and
    /// current premium rate) into the coverage pool and receives a policy
    /// with the specified coverage and duration.
    pub fn purchase_policy(
        env: Env,
        holder: Address,
        covered_contract: Address,
        coverage_amount: i128,
        duration_seconds: u64,
    ) -> u64 {
        holder.require_auth();
        Self::require_initialized(&env);
        Self::require_not_paused(&env);

        // Validate inputs.
        if coverage_amount < MIN_COVERAGE || coverage_amount > MAX_COVERAGE {
            panic_with_error!(&env, Error::InvalidAmount);
        }
        if duration_seconds == 0 || duration_seconds > MAX_POLICY_DURATION {
            panic_with_error!(&env, Error::InvalidDuration);
        }
        if holder == covered_contract {
            panic_with_error!(&env, Error::SelfAssessment);
        }

        let now = env.ledger().timestamp();
        let premium_bps = env.storage().instance().get(&DataKey::PremiumBps).unwrap_or(DEFAULT_PREMIUM_BPS);
        let premium = (coverage_amount * premium_bps) / PREMIUM_BPS_DENOM;

        let id: u64 = env.storage().instance().get(&DataKey::PolicyCount).unwrap_or(0);
        let policy = Policy {
            id,
            holder: holder.clone(),
            covered_contract,
            coverage_amount,
            premium_paid: premium,
            start_time: now,
            expiry_time: now + duration_seconds,
            status: PolicyStatus::Active,
            ledger: env.ledger().sequence(),
        };

        env.storage().persistent().set(&DataKey::Policy(id), &policy);
        env.storage().instance().set(&DataKey::PolicyCount, &(id + 1));

        // Add premium to coverage pool.
        let pool: i128 = env.storage().instance().get(&DataKey::CoveragePool).unwrap_or(0);
        env.storage().instance().set(&DataKey::CoveragePool, &(pool + premium));

        // Track policy by holder.
        let mut holder_policies: Vec<u64> = env
            .storage()
            .persistent()
            .get(&DataKey::HolderPolicies(holder.clone()))
            .unwrap_or_else(|| Vec::new(&env));
        holder_policies.push_back(id);
        env.storage()
            .persistent()
            .set(&DataKey::HolderPolicies(holder.clone()), &holder_policies);

        env.events()
            .publish((symbol_short!("policy"), holder), (id, coverage_amount, premium));

        id
    }

    /// Get a policy by ID.
    pub fn get_policy(env: Env, id: u64) -> Policy {
        env.storage()
            .persistent()
            .get(&DataKey::Policy(id))
            .unwrap_or_else(|| panic_with_error!(&env, Error::PolicyNotFound))
    }

    /// Get all policies for a holder.
    pub fn get_holder_policies(env: Env, holder: Address) -> Vec<u64> {
        env.storage()
            .persistent()
            .get(&DataKey::HolderPolicies(holder))
            .unwrap_or_else(|| Vec::new(&env))
    }

    /// Total number of policies.
    pub fn get_policy_count(env: Env) -> u64 {
        env.storage().instance().get(&DataKey::PolicyCount).unwrap_or(0)
    }

    /// Current coverage pool balance.
    pub fn get_pool_balance(env: Env) -> i128 {
        env.storage().instance().get(&DataKey::CoveragePool).unwrap_or(0)
    }

    /// Check if a policy is still active (not expired, not claimed).
    pub fn is_policy_active(env: Env, id: u64) -> bool {
        let policy: Policy = match env.storage().persistent().get(&DataKey::Policy(id)) {
            Some(p) => p,
            None => return false,
        };
        let now = env.ledger().timestamp();
        if policy.status != PolicyStatus::Active {
            return false;
        }
        if now >= policy.expiry_time {
            return false;
        }
        true
    }

    /// File a claim against a policy. Returns the claim ID.
    pub fn file_claim(
        env: Env,
        claimant: Address,
        policy_id: u64,
        claim_amount: i128,
        evidence: String,
    ) -> u64 {
        claimant.require_auth();
        Self::require_initialized(&env);

        let policy: Policy = env
            .storage()
            .persistent()
            .get(&DataKey::Policy(policy_id))
            .unwrap_or_else(|| panic_with_error!(&env, Error::PolicyNotFound));

        if policy.holder != claimant {
            panic_with_error!(&env, Error::NotPolicyHolder);
        }
        if policy.status != PolicyStatus::Active {
            panic_with_error!(&env, Error::PolicyNotActive);
        }
        let now = env.ledger().timestamp();
        if now >= policy.expiry_time {
            panic_with_error!(&env, Error::PolicyExpired);
        }
        if claim_amount <= 0 || claim_amount > policy.coverage_amount {
            panic_with_error!(&env, Error::InvalidAmount);
        }
        if evidence.len() == 0 || evidence.len() > MAX_EVIDENCE_LEN {
            panic_with_error!(&env, Error::InvalidEvidence);
        }

        let id: u64 = env.storage().instance().get(&DataKey::ClaimCount).unwrap_or(0);
        let claim = Claim {
            id,
            policy_id,
            claimant: claimant.clone(),
            claim_amount,
            evidence,
            status: ClaimStatus::Pending,
            assessor: None,
            filed_at: now,
            reviewed_at: None,
        };

        env.storage().persistent().set(&DataKey::Claim(id), &claim);
        env.storage().instance().set(&DataKey::ClaimCount, &(id + 1));

        // Mark policy as claimed to prevent duplicate claims.
        let mut updated = policy;
        updated.status = PolicyStatus::Claimed;
        env.storage().persistent().set(&DataKey::Policy(policy_id), &updated);

        env.events()
            .publish((symbol_short!("claim"), claimant), (id, policy_id, claim_amount));

        id
    }

    /// Get a claim by ID.
    pub fn get_claim(env: Env, id: u64) -> Claim {
        env.storage()
            .persistent()
            .get(&DataKey::Claim(id))
            .unwrap_or_else(|| panic_with_error!(&env, Error::ClaimNotFound))
    }

    /// Total number of claims.
    pub fn get_claim_count(env: Env) -> u64 {
        env.storage().instance().get(&DataKey::ClaimCount).unwrap_or(0)
    }

    /// Approve a claim. Assessor-only.
    pub fn approve_claim(env: Env, claim_id: u64, assessor: Address) {
        assessor.require_auth();
        Self::require_initialized(&env);

        if !Self::is_assessor(env.clone(), assessor.clone()) {
            panic_with_error!(&env, Error::NotAssessor);
        }

        let mut claim: Claim = env
            .storage()
            .persistent()
            .get(&DataKey::Claim(claim_id))
            .unwrap_or_else(|| panic_with_error!(&env, Error::ClaimNotFound));

        if claim.status != ClaimStatus::Pending {
            panic_with_error!(&env, Error::ClaimAlreadyProcessed);
        }

        // Check pool has enough funds.
        let pool: i128 = env.storage().instance().get(&DataKey::CoveragePool).unwrap_or(0);
        if pool < claim.claim_amount {
            panic_with_error!(&env, Error::InsufficientPool);
        }

        let now = env.ledger().timestamp();
        claim.status = ClaimStatus::Approved;
        claim.assessor = Some(assessor.clone());
        claim.reviewed_at = Some(now);
        env.storage().persistent().set(&DataKey::Claim(claim_id), &claim);

        env.events()
            .publish((symbol_short!("approved"), assessor), claim_id);
    }

    /// Reject a claim. Assessor-only.
    pub fn reject_claim(env: Env, claim_id: u64, assessor: Address) {
        assessor.require_auth();
        Self::require_initialized(&env);

        if !Self::is_assessor(env.clone(), assessor.clone()) {
            panic_with_error!(&env, Error::NotAssessor);
        }

        let mut claim: Claim = env
            .storage()
            .persistent()
            .get(&DataKey::Claim(claim_id))
            .unwrap_or_else(|| panic_with_error!(&env, Error::ClaimNotFound));

        if claim.status != ClaimStatus::Pending {
            panic_with_error!(&env, Error::ClaimAlreadyProcessed);
        }

        let now = env.ledger().timestamp();
        claim.status = ClaimStatus::Rejected;
        claim.assessor = Some(assessor.clone());
        claim.reviewed_at = Some(now);
        env.storage().persistent().set(&DataKey::Claim(claim_id), &claim);

        // Restore policy to active so holder can re-file if desired.
        let mut policy: Policy = env
            .storage()
            .persistent()
            .get(&DataKey::Policy(claim.policy_id))
            .unwrap_or_else(|| panic_with_error!(&env, Error::PolicyNotFound));
        if policy.status == PolicyStatus::Claimed {
            policy.status = PolicyStatus::Active;
            env.storage().persistent().set(&DataKey::Policy(claim.policy_id), &policy);
        }

        env.events()
            .publish((symbol_short!("rejected"), assessor), claim_id);
    }

    /// Mark an approved claim as paid (payout executed off-chain or via
    /// a separate payment contract). Assessor or admin.
    pub fn mark_claim_paid(env: Env, claim_id: u64) {
        Self::require_initialized(&env);
        let admin = Self::read_admin(&env);
        admin.require_auth();

        let mut claim: Claim = env
            .storage()
            .persistent()
            .get(&DataKey::Claim(claim_id))
            .unwrap_or_else(|| panic_with_error!(&env, Error::ClaimNotFound));

        if claim.status != ClaimStatus::Approved {
            panic_with_error!(&env, Error::ClaimAlreadyProcessed);
        }

        // Deduct from pool.
        let pool: i128 = env.storage().instance().get(&DataKey::CoveragePool).unwrap_or(0);
        env.storage().instance().set(&DataKey::CoveragePool, &(pool - claim.claim_amount));

        claim.status = ClaimStatus::Paid;
        env.storage().persistent().set(&DataKey::Claim(claim_id), &claim);

        env.events()
            .publish((symbol_short!("paid"), admin), (claim_id, claim.claim_amount));
    }

    /// Withdraw funds from the coverage pool. Admin-only.
    pub fn withdraw_pool(env: Env, amount: i128) {
        let admin = Self::read_admin(&env);
        admin.require_auth();
        if amount <= 0 {
            panic_with_error!(&env, Error::InvalidAmount);
        }
        let pool: i128 = env.storage().instance().get(&DataKey::CoveragePool).unwrap_or(0);
        if pool < amount {
            panic_with_error!(&env, Error::InsufficientPool);
        }
        env.storage().instance().set(&DataKey::CoveragePool, &(pool - amount));
        env.events().publish((symbol_short!("withdraw"), admin), amount);
    }

    /// Expire policies that have passed their expiry time.
    /// Anyone can call this; it's a maintenance function.
    pub fn expire_stale_policies(env: Env) -> u64 {
        Self::require_initialized(&env);
        let count: u64 = env.storage().instance().get(&DataKey::PolicyCount).unwrap_or(0);
        let now = env.ledger().timestamp();
        let mut expired = 0u64;
        let mut i = 0u64;
        while i < count {
            if let Some(mut policy) = env.storage().persistent().get::<DataKey, Policy>(&DataKey::Policy(i)) {
                if policy.status == PolicyStatus::Active && now >= policy.expiry_time {
                    policy.status = PolicyStatus::Expired;
                    env.storage().persistent().set(&DataKey::Policy(i), &policy);
                    expired += 1;
                }
            }
            i += 1;
        }
        if expired > 0 {
            env.events().publish((symbol_short!("expired"),), expired);
        }
        expired
    }

    // ----- internal helpers -----

    fn read_admin(env: &Env) -> Address {
        env.storage()
            .instance()
            .get(&DataKey::Admin)
            .unwrap_or_else(|| panic_with_error!(env, Error::NotInitialized))
    }

    fn require_initialized(env: &Env) {
        if !env.storage().instance().has(&DataKey::Admin) {
            panic_with_error!(env, Error::NotInitialized);
        }
    }

    fn require_not_paused(env: &Env) {
        let paused: bool = env.storage().instance().get(&DataKey::Paused).unwrap_or(false);
        if paused {
            panic_with_error!(env, Error::Paused);
        }
    }
}

#[cfg(test)]
mod test;
