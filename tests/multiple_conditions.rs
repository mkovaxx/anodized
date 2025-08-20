use anodized::contract;

struct SafeBuffer<T> {
    items: Vec<T>,
    initialized: bool,
    locked: bool,
}

impl<T> SafeBuffer<T> {
    #[contract(
        maintains: self.items.len() <= self.items.capacity(),
        requires: self.initialized,
        requires: !self.locked,
        requires: index < self.items.len(),
    )]
    fn get_element(&self, index: usize) -> &T {
        &self.items[index]
    }
}

#[test]
fn test_get_element_success() {
    let buffer = SafeBuffer {
        items: vec![10, 20, 30],
        initialized: true,
        locked: false,
    };
    assert_eq!(*buffer.get_element(1), 20);
}

#[test]
#[should_panic(expected = "Precondition failed: ! self.locked")]
fn test_get_element_panics_when_locked() {
    let buffer = SafeBuffer {
        items: vec![10, 20, 30],
        initialized: true,
        locked: true, // This will violate the precondition.
    };
    buffer.get_element(1);
}

#[test]
#[should_panic(expected = "Precondition failed: index < self.items.len()")]
fn test_get_element_panics_on_out_of_bounds() {
    let buffer = SafeBuffer {
        items: vec![10, 20, 30],
        initialized: true,
        locked: false,
    };
    buffer.get_element(5); // This will violate the precondition.
}
