use anodized::spec;

#[cfg(not(feature = "backend-nightly-contracts"))]
#[spec(
    requires: {
        // Just a longer way of writing `true` :)
        let x = 5;
        x > 0
    },
    maintains: {
        let length = vec.len();
        length < 100
    },
    captures: {
        let snapshot = vec.clone();
        snapshot.len()
    } as old_len,
    ensures: {
        let length = vec.len();
        length > old_len
    },
)]
fn function_with_blocks(vec: &mut Vec<i32>) {
    vec.push(42);
}

#[cfg(feature = "backend-nightly-contracts")]
#[spec(
    requires: {
        // Just a longer way of writing `true` :)
        let x = 5;
        x > 0
    },
    maintains: {
        let length = vec.len();
        length < 100
    },
    ensures: {
        let length = vec.len();
        length > 0
    },
)]
fn function_with_blocks(vec: &mut Vec<i32>) {
    vec.push(42);
}

#[test]
fn test_block_expressions() {
    let mut vec = vec![1, 2, 3];
    function_with_blocks(&mut vec);
}
