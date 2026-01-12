
use anodized::spec;

#[spec]
pub trait TestTrait {

    fn current(&self) -> u32;

    #[spec(
        requires: {
            // Just a longer way of writing `true` :)
            let x = 5;
            x > 0
        },
        captures: {
            self.current()
        } as old_val,
        ensures: {
            let new_val = self.current();
            new_val > old_val
        },
    )]
    fn do_something(&self, x: u32) -> u32;
}

struct TestStruct;

impl TestTrait for TestStruct {
    fn do_something(&self, x: u32) -> u32 {
        x * 2
    }
}

#[test]
fn basic_trait_test() {
    let test = TestStruct;

    assert_eq!(test.do_something(500), 1000);
}