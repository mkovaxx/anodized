use anodized::spec;

#[spec(
    requires: vec.len() < vec.capacity() || vec.capacity() == 0,
    maintains: vec.len() <= vec.capacity(),
)]
fn push_checked<T>(vec: &mut Vec<T>, value: T) {
    vec.push(value);
}

fn main() {
    let mut vec = Vec::with_capacity(2);

    // These calls satisfy the spec
    push_checked(&mut vec, 1);
    push_checked(&mut vec, 2);

    println!("Vec after valid pushes: {:?}", vec);

    // This call violates the precondition (no capacity left)
    push_checked(&mut vec, 3);

    println!("Vec after violating push: {:?}", vec);
}
