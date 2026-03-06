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

// Multiple clauses in requires with comments on individual clauses
#[spec(
    requires: [
            // x must be positive
        x > 0,
        // y must be positive
        y > 0,
                // z must be positive
        z > 0,
    ],
    ensures: *output > 0,
)]
fn requires_multiple_clauses_with_comments(x: i32, y: i32, z: i32) -> i32 {
    todo!()
}

// Multiple clauses in ensures with comments on individual clauses
#[spec(
    requires: x > 0,
    ensures: [
                // result is greater than input
        *output > x,
        // result is less than 100
        *output < 100,
    ],
)]
fn ensures_multiple_clauses_with_comments(x: i32) -> i32 {
    todo!()
}

// Multiple clauses in maintains with comments on individual clauses
#[spec(
    maintains: [
        // count is non-negative
        self.count >= 0,
            // state is active
        self.active,
    ],
    ensures: *output == self.count,
)]
fn maintains_multiple_clauses_with_comments(&self) -> i32 {
    todo!()
}
