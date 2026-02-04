use std::panic::catch_unwind;

use anodized::spec;

#[spec(
    requires: *balance >= amount,
    maintains: *balance >= 0,
    captures: *balance as initial_balance,
    binds: (new_balance, receipt_amount),
    ensures: [
        *new_balance == initial_balance - amount,
        *receipt_amount == amount,
        *balance == *new_balance,
    ],
)]
fn withdraw(balance: &mut u64, amount: u64) -> (u64, u64) {
    *balance -= amount;
    (*balance, amount)
}

fn main() {
    let mut balance = 100;

    // Valid withdrawal
    let (new_balance, receipt) = withdraw(&mut balance, 30);
    println!("Withdrew {}, new balance: {}", receipt, new_balance);

    // Another valid withdrawal
    let (new_balance, receipt) = withdraw(&mut balance, 50);
    println!("Withdrew {}, new balance: {}", receipt, new_balance);

    // This violates the precondition (not enough balance)
    assert!(
        catch_unwind(|| {
            let mut balance = 20;
            let (new_balance, receipt) = withdraw(&mut balance, 100);
            println!("Withdrew {}, new balance: {}", receipt, new_balance);
        })
        .is_err()
    );

    println!("Error was expected above, this should still print");
}
