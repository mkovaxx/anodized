
use anodized::spec;

#[spec]
pub trait TestTrait {

    /// Returns a current value
    fn current(&self) -> u32;

    /// Does something
    #[spec(
        requires: x > 0,
        captures: self.current() as old_val,
        ensures: *output > old_val,
    )]
    fn do_something(&self, x: u32) -> u32 {
        x * 2
    }
}

struct TestStruct(u32);

#[spec]
impl TestTrait for TestStruct {
    fn current(&self) -> u32 {
        self.0
    }
    #[spec(
        maintains: self.0 == 3,
        ensures: *output > self.0,
    )]
    #[inline(never)]
    fn do_something(&self, x: u32) -> u32 {
        x * self.0
    }
}

struct TestStructConst;

#[spec]
impl TestTrait for TestStructConst {
    fn current(&self) -> u32 {
        0
    }
}

#[test]
fn basic_trait_test() {
    // Tests an impl of a trait with a spec, where there is a spec on both the implementation and on the trait interface
    let test = TestStruct(3);
    assert_eq!(test.do_something(500), 1500);

    // Tests a default method implementation coming from a trait
    let test = TestStructConst;
    assert_eq!(test.do_something(500), 1000);
}
