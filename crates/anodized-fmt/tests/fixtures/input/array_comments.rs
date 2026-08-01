use anodized::spec;

// Comments in captures array
#[spec(
        // Comment on requires clause
    requires: active,
           // Comment on captures clause array
    captures: [
               // Capture 1st
        [first , second , third] = values,
    // Capture 2nd
        State { active , count } = state.clone(),
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
