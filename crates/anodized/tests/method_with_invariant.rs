use anodized::spec;

struct Counter {
    count: u32,
    capacity: u32,
}

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

#[cfg(not(feature = "backend-no-checks"))]
#[test]
#[should_panic(expected = "Post-invariant failed: self.count <= self.capacity")]
fn increment_violates_invariant() {
    let mut c = Counter {
        count: 10,
        capacity: 10,
    };
    c.increment(); // This will make count 11, violating the invariant on exit.
}

#[cfg(not(feature = "backend-no-checks"))]
#[test]
#[should_panic(expected = "Pre-invariant failed: self.count <= self.capacity")]
fn increment_violates_pre_invariant() {
    let mut c = Counter {
        count: 11,
        capacity: 10, // count > capacity, violates pre-invariant
    };
    c.increment();
}
