use anodized::spec;

#[spec]
pub trait TestTrait {
    /// Returns a current value
    fn current(&self) -> u32;

    /// Has no default
    #[spec(
        requires: x > 0,
        captures: self.current() as old_val,
        ensures: *output > old_val,
    )]
    fn add_to(&self, x: u32) -> u32;

    /// Has a default
    #[spec(
        requires: x > 0,
        captures: self.current() as old_val,
        ensures: *output > old_val,
    )]
    fn mul_by(&self, x: u32) -> u32 {
        x * 2
    }
}

struct TestStruct(u32);

#[spec]
impl TestTrait for TestStruct {
    fn current(&self) -> u32 {
        self.0
    }

    fn add_to(&self, x: u32) -> u32 {
        x + self.current()
    }

    #[inline(never)]
    fn mul_by(&self, x: u32) -> u32 {
        x * self.0
    }
}

struct TestStructConst;

#[spec]
impl TestTrait for TestStructConst {
    fn current(&self) -> u32 {
        0
    }

    fn add_to(&self, x: u32) -> u32 {
        x + self.current()
    }
}

#[test]
fn should_succeed() {
    // Tests an impl of a trait with a spec
    let test = TestStruct(3);
    assert_eq!(test.add_to(500), 503);
    assert_eq!(test.mul_by(500), 1500);

    // Tests the default method implementation coming from the trait
    let test = TestStructConst;
    assert_eq!(test.add_to(500), 500);
    assert_eq!(test.mul_by(500), 1000);
}
