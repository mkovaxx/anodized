use anodized::spec;

#[spec(
    requires: *balance >= amount,
    maintains: *balance >= 0,
    captures: initial_balance = *balance,
    ensures: *balance == initial_balance - amount,
)]
fn withdraw(balance: &mut u64, amount: u64) {
    *balance -= amount;
}
