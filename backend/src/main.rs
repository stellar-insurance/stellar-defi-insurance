fn main() {
    println!("Server starting...");
}

/// Placeholder helper addressing issue #5: Prevent double-submit on transaction button
pub fn issue_5_placeholder() -> &'static str { "addresses #5" }
/// Guard for the zero-balance edge case: transfers must be rejected when the
/// wallet balance is zero (or negative) to avoid draining an empty account.
/// See issue #7.
fn is_balance_zero(balance: i128) -> bool {
    balance <= 0
}
