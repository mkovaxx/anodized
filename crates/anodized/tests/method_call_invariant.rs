use anodized::spec;

#[allow(dead_code)]
struct Validator {
    valid: bool,
}

impl Validator {
    fn is_valid(&self) -> bool {
        self.valid
    }

    #[spec(
        maintains: self.is_valid(),
    )]
    fn set_validity(&mut self, new_validity: bool) {
        self.valid = new_validity;
    }
}

#[cfg(not(any(feature = "backend-no-checks", feature = "backend-no-panic")))]
#[test]
#[should_panic(expected = "Post-invariant failed: self.is_valid()")]
fn violates_post_invariant() {
    let mut v = Validator { valid: true };
    // This will violate the invariant on exit.
    v.set_validity(false);
}

#[cfg(not(any(feature = "backend-no-checks", feature = "backend-no-panic")))]
#[test]
#[should_panic(expected = "Pre-invariant failed: self.is_valid()")]
fn violates_pre_invariant() {
    let mut v = Validator { valid: false };
    // This violates the invariant on entry.
    v.set_validity(true);
}
