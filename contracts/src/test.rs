#![cfg(test)]
//! Unit tests for the DeFi insurance contract.

use super::*;
use soroban_sdk::{
    symbol_short,
    testutils::{Address as _, Events, Ledger},
    vec, Address, Env, IntoVal, String,
};

fn setup() -> (Env, InsuranceContractClient<'static>, Address) {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(InsuranceContract, ());
    let client = InsuranceContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    client.initialize(&admin);
    (env, client, admin)
}

fn make_policy(
    env: &Env,
    client: &InsuranceContractClient,
    holder: &Address,
    covered: &Address,
    coverage: i128,
    duration: u64,
) -> u64 {
    client.purchase_policy(holder, covered, &coverage, &duration)
}

#[test]
fn initialize_sets_admin() {
    let (_env, client, admin) = setup();
    assert_eq!(client.get_admin(), admin);
    assert_eq!(client.is_paused(), false);
    assert_eq!(client.get_policy_count(), 0);
    assert_eq!(client.get_pool_balance(), 0);
    assert_eq!(client.get_premium_rate(), 500); // 5%
}

#[test]
#[should_panic(expected = "Error(Contract, #1)")]
fn initialize_twice_panics() {
    let (_env, client, admin) = setup();
    client.initialize(&admin);
}

#[test]
fn purchase_policy_happy_path() {
    let (env, client, _admin) = setup();
    let holder = Address::generate(&env);
    let covered = Address::generate(&env);

    let id = make_policy(&env, &client, &holder, &covered, 10_000, 3600);
    assert_eq!(id, 0);
    assert_eq!(client.get_policy_count(), 1);

    let policy = client.get_policy(&0);
    assert_eq!(policy.holder, holder);
    assert_eq!(policy.coverage_amount, 10_000);
    assert_eq!(policy.status, PolicyStatus::Active);
    // Premium = 10000 * 500 / 10000 = 500
    assert_eq!(policy.premium_paid, 500);
    assert_eq!(client.get_pool_balance(), 500);
}

#[test]
fn purchase_multiple_policies() {
    let (env, client, _admin) = setup();
    let holder = Address::generate(&env);
    let covered = Address::generate(&env);

    let id0 = make_policy(&env, &client, &holder, &covered, 10_000, 3600);
    let id1 = make_policy(&env, &client, &holder, &covered, 20_000, 7200);
    assert_eq!(id0, 0);
    assert_eq!(id1, 1);
    assert_eq!(client.get_policy_count(), 2);
    // Pool = 500 + 1000 = 1500
    assert_eq!(client.get_pool_balance(), 1500);

    let holder_policies = client.get_holder_policies(&holder);
    assert_eq!(holder_policies.len(), 2);
    assert_eq!(holder_policies.get(0).unwrap(), 0);
    assert_eq!(holder_policies.get(1).unwrap(), 1);
}

#[test]
#[should_panic(expected = "Error(Contract, #5)")]
fn purchase_rejects_below_min_coverage() {
    let (env, client, _admin) = setup();
    let holder = Address::generate(&env);
    let covered = Address::generate(&env);
    make_policy(&env, &client, &holder, &covered, 10, 3600);
}

#[test]
#[should_panic(expected = "Error(Contract, #6)")]
fn purchase_rejects_zero_duration() {
    let (env, client, _admin) = setup();
    let holder = Address::generate(&env);
    let covered = Address::generate(&env);
    make_policy(&env, &client, &holder, &covered, 10_000, 0);
}

#[test]
#[should_panic(expected = "Error(Contract, #4)")]
fn purchase_blocked_when_paused() {
    let (env, client, _admin) = setup();
    let holder = Address::generate(&env);
    let covered = Address::generate(&env);
    client.pause();
    make_policy(&env, &client, &holder, &covered, 10_000, 3600);
}

#[test]
fn pause_unpause_cycle() {
    let (_env, client, _admin) = setup();
    client.pause();
    assert!(client.is_paused());
    client.unpause();
    assert!(!client.is_paused());
}

#[test]
fn set_premium_rate() {
    let (_env, client, _admin) = setup();
    client.set_premium_rate(&1000); // 10%
    assert_eq!(client.get_premium_rate(), 1000);
}

#[test]
fn grant_and_revoke_assessor() {
    let (env, client, _admin) = setup();
    let assessor = Address::generate(&env);
    assert!(!client.is_assessor(&assessor));
    client.grant_assessor(&assessor);
    assert!(client.is_assessor(&assessor));
    client.revoke_assessor(&assessor);
    assert!(!client.is_assessor(&assessor));
}

#[test]
fn file_claim_happy_path() {
    let (env, client, _admin) = setup();
    let holder = Address::generate(&env);
    let covered = Address::generate(&env);
    make_policy(&env, &client, &holder, &covered, 10_000, 3600);

    let claim_id = client.file_claim(
        &holder,
        &0,
        &5_000,
        &String::from_str(&env, "Smart contract exploit: reentrancy attack on Covered contract. Evidence: tx hash 0xabc..."),
    );
    assert_eq!(claim_id, 0);
    assert_eq!(client.get_claim_count(), 1);

    let claim = client.get_claim(&0);
    assert_eq!(claim.policy_id, 0);
    assert_eq!(claim.claim_amount, 5_000);
    assert_eq!(claim.status, ClaimStatus::Pending);

    // Policy should be marked as Claimed.
    let policy = client.get_policy(&0);
    assert_eq!(policy.status, PolicyStatus::Claimed);
}

#[test]
#[should_panic(expected = "Error(Contract, #12)")]
fn file_claim_by_non_holder() {
    let (env, client, _admin) = setup();
    let holder = Address::generate(&env);
    let covered = Address::generate(&env);
    let stranger = Address::generate(&env);
    make_policy(&env, &client, &holder, &covered, 10_000, 3600);
    client.file_claim(&stranger, &0, &5_000, &String::from_str(&env, "x"));
}

#[test]
#[should_panic(expected = "Error(Contract, #5)")]
fn file_claim_exceeds_coverage() {
    let (env, client, _admin) = setup();
    let holder = Address::generate(&env);
    let covered = Address::generate(&env);
    make_policy(&env, &client, &holder, &covered, 10_000, 3600);
    client.file_claim(&holder, &0, &20_000, &String::from_str(&env, "x"));
}

#[test]
#[should_panic(expected = "Error(Contract, #9)")]
fn file_claim_on_expired_policy() {
    let (env, client, _admin) = setup();
    let holder = Address::generate(&env);
    let covered = Address::generate(&env);
    make_policy(&env, &client, &holder, &covered, 10_000, 3600);

    // Advance time past expiry.
    env.ledger().with_mut(|l| l.timestamp = l.timestamp + 7200);
    client.file_claim(&holder, &0, &5_000, &String::from_str(&env, "x"));
}

#[test]
fn approve_and_pay_claim() {
    let (env, client, admin) = setup();
    let holder = Address::generate(&env);
    let covered = Address::generate(&env);
    let assessor = Address::generate(&env);

    client.grant_assessor(&assessor);
    // Purchase enough policies to build pool: each 10_000 coverage = 500 premium
    // Need at least 5000 in pool, so buy 10+ policies
    for _ in 0..11 {
        make_policy(&env, &client, &holder, &covered, 10_000, 3600);
    }
    // Pool = 11 * 500 = 5500
    assert_eq!(client.get_pool_balance(), 5500);

    // File claim on policy #0 for 5000
    client.file_claim(&holder, &0, &5_000, &String::from_str(&env, "Exploit evidence"));

    // Approve
    client.approve_claim(&0, &assessor);
    let claim = client.get_claim(&0);
    assert_eq!(claim.status, ClaimStatus::Approved);
    assert_eq!(claim.assessor.unwrap(), assessor);

    // Mark as paid (admin)
    client.mark_claim_paid(&0);
    let claim = client.get_claim(&0);
    assert_eq!(claim.status, ClaimStatus::Paid);
    // Pool after payout: 5500 - 5000 = 500
    assert_eq!(client.get_pool_balance(), 500);
}

#[test]
fn reject_claim_restores_policy() {
    let (env, client, _admin) = setup();
    let holder = Address::generate(&env);
    let covered = Address::generate(&env);
    let assessor = Address::generate(&env);

    client.grant_assessor(&assessor);
    make_policy(&env, &client, &holder, &covered, 10_000, 3600);
    client.file_claim(&holder, &0, &5_000, &String::from_str(&env, "Evidence"));

    client.reject_claim(&0, &assessor);
    let claim = client.get_claim(&0);
    assert_eq!(claim.status, ClaimStatus::Rejected);

    // Policy restored to Active.
    let policy = client.get_policy(&0);
    assert_eq!(policy.status, PolicyStatus::Active);
}

#[test]
#[should_panic(expected = "Error(Contract, #16)")]
fn approve_claim_by_non_assessor() {
    let (env, client, _admin) = setup();
    let holder = Address::generate(&env);
    let covered = Address::generate(&env);
    let fake = Address::generate(&env);
    make_policy(&env, &client, &holder, &covered, 10_000, 3600);
    client.file_claim(&holder, &0, &5_000, &String::from_str(&env, "x"));
    client.approve_claim(&0, &fake);
}

#[test]
fn expire_stale_policies() {
    let (env, client, _admin) = setup();
    let holder = Address::generate(&env);
    let covered = Address::generate(&env);

    make_policy(&env, &client, &holder, &covered, 10_000, 3600);
    make_policy(&env, &client, &holder, &covered, 10_000, 7200);

    // Advance time past first policy only.
    env.ledger().with_mut(|l| l.timestamp = l.timestamp + 4000);

    let expired = client.expire_stale_policies();
    assert_eq!(expired, 1);

    let p0 = client.get_policy(&0);
    assert_eq!(p0.status, PolicyStatus::Expired);
    let p1 = client.get_policy(&1);
    assert_eq!(p1.status, PolicyStatus::Active);
}

#[test]
fn is_policy_active_checks() {
    let (env, client, _admin) = setup();
    let holder = Address::generate(&env);
    let covered = Address::generate(&env);
    make_policy(&env, &client, &holder, &covered, 10_000, 3600);

    assert!(client.is_policy_active(&0));

    env.ledger().with_mut(|l| l.timestamp = l.timestamp + 4000);
    assert!(!client.is_policy_active(&0));
}

#[test]
fn withdraw_pool_admin() {
    let (env, client, _admin) = setup();
    let holder = Address::generate(&env);
    let covered = Address::generate(&env);
    make_policy(&env, &client, &holder, &covered, 10_000, 3600);
    // Pool = 500
    assert_eq!(client.get_pool_balance(), 500);

    client.withdraw_pool(&200);
    assert_eq!(client.get_pool_balance(), 300);
}

#[test]
#[should_panic(expected = "Error(Contract, #14)")]
fn withdraw_pool_insufficient() {
    let (_env, client, _admin) = setup();
    client.withdraw_pool(&100);
}
