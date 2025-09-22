use anodized::spec;

// Test simple identifier capturing with auto-generated alias
#[spec(
    captures: count,
    ensures: old_count <= *output,
)]
fn increment_counter(count: u32) -> u32 {
    count + 1
}

// Struct for testing
struct Container {
    value: i32,
    counter: u32,
    items: Vec<String>,
    capacity: usize,
}

impl Container {
    fn new() -> Self {
        Container {
            value: 0,
            counter: 0,
            items: Vec::new(),
            capacity: 10,
        }
    }

    fn is_valid(&self) -> bool {
        self.counter < 100
    }

    #[spec(
        captures: self.value as initial_value,
        ensures: self.value == initial_value + amount,
    )]
    fn add_to_value(&mut self, amount: i32) {
        self.value += amount;
    }

    #[spec(
        captures: self.items.clone() as original_items,
        ensures: self.items.len() == original_items.len() || self.items.len() == original_items.len() + 1,
    )]
    fn maybe_push(&mut self, item: String, should_push: bool) {
        if should_push {
            self.items.push(item);
        }
    }

    #[spec(
        captures: [
            self.items.len() as original_len,
            self.capacity as original_cap,
        ],
        ensures: [
            self.items.len() == original_len + 1,
            self.capacity >= original_cap,
        ],
    )]
    fn push_item(&mut self, item: String) {
        if self.items.len() == self.capacity {
            self.capacity *= 2;
        }
        self.items.push(item);
    }

    #[spec(
        requires: self.is_valid(),
        captures: self.counter as old_counter,
        ensures: self.counter == old_counter + 1,
    )]
    fn increment_if_valid(&mut self) {
        self.counter += 1;
    }
}

#[test]
fn test_simple_capture_with_auto_alias() {
    assert_eq!(increment_counter(5), 6);
    assert_eq!(increment_counter(0), 1);
}

#[test]
fn test_capture_with_explicit_alias() {
    let mut container = Container::new();
    container.value = 10;
    container.add_to_value(5);
    assert_eq!(container.value, 15);
}

#[test]
fn test_multiple_captures() {
    let mut container = Container::new();

    // Add items up to capacity
    for i in 0..10 {
        container.push_item(format!("item_{}", i));
    }
    assert_eq!(container.items.len(), 10);
    assert_eq!(container.capacity, 10);

    // This should trigger capacity doubling
    container.push_item("item_10".to_string());
    assert_eq!(container.items.len(), 11);
    assert_eq!(container.capacity, 20);
}

#[test]
fn test_captures_with_preconditions() {
    let mut container = Container::new();
    container.counter = 50;

    container.increment_if_valid();
    assert_eq!(container.counter, 51);
}

#[cfg(not(feature = "backend-no-checks"))]
#[test]
#[should_panic(expected = "Postcondition failed")]
fn test_capture_postcondition_failure() {
    #[spec(
        captures: *value as old_value,
        ensures: *value == old_value + 10,
    )]
    fn bad_increment(value: &mut i32) {
        // Wrong! Should add 10
        *value += 5
    }

    let mut val = 5;
    bad_increment(&mut val);
}

#[cfg(not(feature = "backend-no-checks"))]
#[test]
#[should_panic(expected = "Precondition failed")]
fn test_precondition_runs_before_captures() {
    struct TestStruct {
        counter: u32,
    }

    impl TestStruct {
        #[spec(
            requires: self.counter < 100,
            captures: self.counter as old_counter,
            ensures: self.counter == old_counter + 1,
        )]
        fn increment(&mut self) {
            self.counter += 1;
        }
    }

    let mut test = TestStruct { counter: 100 };
    // Should panic on precondition, not reach captures
    test.increment();
}

#[test]
fn test_explicit_clone_for_non_copy_types() {
    let mut container = Container::new();
    container.items.push("first".to_string());
    container.items.push("second".to_string());

    // Should not push
    container.maybe_push("third".to_string(), false);
    assert_eq!(container.items.len(), 2);

    // Should push
    container.maybe_push("third".to_string(), true);
    assert_eq!(container.items.len(), 3);
}
