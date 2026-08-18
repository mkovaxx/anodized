#![allow(unused)]
use anodized::spec;

struct Counter {
    count: u32,
    capacity: u32,
}

#[spec]
impl Counter {
    #[spec(
        maintains: self.count <= self.capacity,
    )]
    fn increment(&mut self) {
        self.count += 1;
    }
}

#[test]
fn increment_success() {
    let mut c = Counter {
        count: 5,
        capacity: 10,
    };
    c.increment();
}

#[cfg(all(anodized_print, anodized_panic))]
#[test]
#[should_panic(expected = "postcondition failed")]
fn increment_violates_invariant() {
    let mut c = Counter {
        count: 10,
        capacity: 10,
    };
    // This will make count 11, violating the invariant on exit.
    c.increment();
}

#[cfg(all(anodized_print, anodized_panic))]
#[test]
#[should_panic(expected = "precondition failed")]
fn increment_violates_pre_invariant() {
    let mut c = Counter {
        count: 11,
        // count > capacity, violates pre-invariant
        capacity: 10,
    };
    c.increment();
}
