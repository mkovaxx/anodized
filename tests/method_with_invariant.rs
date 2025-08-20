use anodized::contract;

struct Counter {
    count: u32,
    capacity: u32,
}

impl Counter {
    #[contract(
        maintains: self.count <= self.capacity,
    )]
    fn increment(&mut self) {
        self.count += 1;
    }
}

#[test]
fn test_increment_success() {
    let mut c = Counter {
        count: 5,
        capacity: 10,
    };
    c.increment();
    assert_eq!(c.count, 6);
}

#[test]
#[should_panic(expected = "Postcondition failed: self.count <= self.capacity")]
fn test_increment_violates_invariant() {
    let mut c = Counter {
        count: 10,
        capacity: 10,
    };
    c.increment(); // This will make count 11, violating the invariant on exit.
}
