use anodized::spec;

#[spec(
    requires: [
        part >= 0.0,
        part <= whole,
        whole > 0.0,
    ],
    ensures: [
        *output >= 0.0,
        *output <= 100.0,
    ],
)]
fn calculate_percentage(part: f64, whole: f64) -> f64 {
    100.0 * part / whole
}

fn main() {
    // This call satisfies the spec and runs fine.
    println!("25 out of 100 = {}%", calculate_percentage(25.0, 100.0));

    // This call violates the precondition and will print an error.
    println!("10 out of 0 = {}%", calculate_percentage(10.0, 0.0));
}
