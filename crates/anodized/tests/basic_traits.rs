
use anodized::spec;

#[spec]
pub trait TestTrait {

    fn current(&self) -> u32;

    #[spec(
        requires: {
            x > 0
        },
        captures: {
            self.current()
        } as old_val,
        ensures: {
            *output > old_val
        },
    )]
    fn do_something(&self, x: u32) -> u32;
}

struct TestStruct;

#[spec]
impl TestTrait for TestStruct {
    fn current(&self) -> u32 {
        0
    }
    fn do_something(&self, x: u32) -> u32 {
        x * 2
    }
}

#[test]
fn basic_trait_test() {
    let test = TestStruct;

    assert_eq!(test.do_something(500), 1000);
}