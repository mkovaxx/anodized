use anodized::spec;

// Following cases are not supported and skipped by the
// current anodized-fmt

// Block expressions with comments
#[spec(
    requires: {
        // Just a longer way of writing `true` :)
        let x = a;
        x > 0
    },
            // Result is positive
    ensures: *output > 0,
)]
fn block_expr(a: i32, b: i32) -> i32 {
    todo!()
}

// Inline comments
#[spec(
          // Requires must be positive
    requires: x > 0,      // Inline comment for requires
    ensures: *output > 0,     // Inline comment for ensures
)]
fn inline_comments(x: i32) -> i32 {
    todo!()
}

// Comments in nested structures
#[spec(
        // Comment on requires clause
    requires: active,
           // Comment on captures clause array
    captures: [
               // Capture 1st
        values as [first , second , third],
    // Capture 2nd
        state.clone() as State { active , count },
    ],
    ensures: first + second + third == count,
)]
fn capture_multiple(values: [i32; 3], state: &State) -> bool {
    todo!()
}
