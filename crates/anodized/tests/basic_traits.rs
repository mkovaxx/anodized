use anodized::spec;

#[spec]
pub trait TestTrait {
    /// Returns a current value
    fn current(&self) -> i32;

    /// Has no default
    #[spec(
        requires: x > 0,
        captures: self.current() as old_val,
        ensures: *output > old_val,
    )]
    fn add_to(&self, x: i32) -> i32;

    /// Has a default
    #[spec(
        requires: x > 0,
        captures: self.current() as old_val,
        ensures: *output > old_val,
    )]
    fn mul_by(&self, x: i32) -> i32 {
        x * 2
    }
}

struct TestStruct(i32);

#[spec]
impl TestTrait for TestStruct {
    fn current(&self) -> i32 {
        self.0
    }

    fn add_to(&self, x: i32) -> i32 {
        x + self.current()
    }

    #[inline(never)]
    fn mul_by(&self, x: i32) -> i32 {
        x * self.0
    }
}

struct TestStructConst;

#[spec]
impl TestTrait for TestStructConst {
    fn current(&self) -> i32 {
        0
    }

    fn add_to(&self, x: i32) -> i32 {
        x + self.current()
    }
}

#[test]
fn should_succeed() {
    let test = TestStruct(3);
    assert_eq!(test.add_to(500), 503);
    assert_eq!(test.mul_by(500), 1500);

    let test = TestStructConst;
    assert_eq!(test.add_to(500), 500);
    assert_eq!(test.mul_by(500), 1000);
}

#[cfg(feature = "runtime-check-and-panic")]
#[test]
#[should_panic(expected = "Precondition failed: x > 0")]
fn should_fail_add_to() {
    let test = TestStruct(3);
    assert_eq!(test.add_to(0), 3);
}

#[cfg(feature = "runtime-check-and-panic")]
#[test]
#[should_panic(expected = "Postcondition failed: | output | * output > old_val")]
fn should_fail_negative_mul_by() {
    let test = TestStruct(-3);
    assert_eq!(test.mul_by(1), 3);
}
