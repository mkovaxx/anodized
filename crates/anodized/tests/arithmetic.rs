use anodized::arithmetic::int;
use anodized::spec;

#[spec(
    requires: n > 0,
    ensures: |output| output == 1,
)]
pub fn collatz(mut n: int) -> int {
    while n > 1 {
        n = f(&n);
    }
    n
}

fn f(n: &int) -> int {
    if n % 2 == 0 { n / 2 } else { 3 * n + 1 }
}
