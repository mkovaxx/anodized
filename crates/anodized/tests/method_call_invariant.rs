#![allow(unused)]
use anodized::spec;

#[allow(dead_code)]
struct Validator {
    valid: bool,
}

#[spec]
impl Validator {
    fn is_valid(&self) -> bool {
        self.valid
    }

    #[spec(
        maintains: self.is_valid(),
    )]
    #[allow(unused)]
    fn set_validity(&mut self, new_validity: bool) {
        self.valid = new_validity;
    }
}

#[cfg(all(anodized_print, anodized_panic))]
#[test]
#[should_panic(expected = "postcondition failed")]
fn violates_post_invariant() {
    let mut v = Validator { valid: true };
    // This will violate the invariant on exit.
    v.set_validity(false);
}

#[cfg(all(anodized_print, anodized_panic))]
#[test]
#[should_panic(expected = "precondition failed")]
fn violates_pre_invariant() {
    let mut v = Validator { valid: false };
    // This violates the invariant on entry.
    v.set_validity(true);
}
