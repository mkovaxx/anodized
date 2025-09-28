use anodized::spec;

struct SafeBuffer<T> {
    items: Vec<T>,
    initialized: bool,
    locked: bool,
}

impl<T> SafeBuffer<T> {
    #[spec(
        requires: [
            self.initialized,
            !self.locked,
            index < self.items.len(),
        ],
        maintains: self.items.len() <= self.items.capacity(),
    )]
    fn get_element(&self, index: usize) -> &T {
        &self.items[index]
    }
}

#[test]
fn get_element_success() {
    let buffer = SafeBuffer {
        items: vec![10, 20, 30],
        initialized: true,
        locked: false,
    };
    buffer.get_element(1);
}

#[cfg(feature = "check-and-panic")]
#[test]
#[should_panic(expected = "Precondition failed: ! self.locked")]
fn get_element_panics_when_locked() {
    let buffer = SafeBuffer {
        items: vec![10, 20, 30],
        initialized: true,
        locked: true, // This will violate the precondition.
    };
    buffer.get_element(1);
}

#[cfg(feature = "check-and-panic")]
#[test]
#[should_panic(expected = "Precondition failed: index < self.items.len()")]
fn get_element_panics_on_out_of_bounds() {
    let buffer = SafeBuffer {
        items: vec![10, 20, 30],
        initialized: true,
        locked: false,
    };
    // This will violate the precondition.
    buffer.get_element(5);
}
