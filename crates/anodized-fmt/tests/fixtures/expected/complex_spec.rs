use anodized::spec;

#[spec(
    requires: *balance >= amount,
    maintains: *balance >= 0,
    captures: *balance as initial_balance,
    ensures: *balance == initial_balance - amount,
)]
fn withdraw(balance: &mut u64, amount: u64) {
    *balance -= amount;
}
