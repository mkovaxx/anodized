#![allow(unused)]

use anodized::spec;

#[spec]
trait Trait<T: Ord + Copy> {
    #[spec(requires: left < right)]
    fn func_1(input @ (left, right): (i32, i32)) {
        unimplemented!()
    }

    #[spec(requires: lower < upper)]
    fn func_2(Bounds { lower, upper }: Bounds<T>) {
        unimplemented!()
    }

    #[spec(
        requires: lower < upper,
        // NOTE: `T: Copy` is needed for this to work at the moment.
        ensures: [
            *output >= lower,
            *output < upper,
        ],
    )]
    fn func_3((input, Bounds { lower, upper }): (T, Bounds<T>)) -> T {
        unimplemented!()
    }
}

struct Bounds<T> {
    lower: T,
    upper: T,
}

struct Type<T> {
    _phantom: std::marker::PhantomData<T>,
}

#[spec]
impl<T: Ord + Copy> Trait<T> for Type<T> {
    fn func_1(_: (i32, i32)) {}
    fn func_2(_: Bounds<T>) {}
    fn func_3((i, _): (T, Bounds<T>)) -> T {
        i
    }
}

#[cfg(all(anodized_print, anodized_panic))]
#[test]
#[should_panic(expected = r#"precondition failed:
    left < right"#)]
fn test_1() {
    let _ = Type::<i32>::func_1((42, 1));
}

#[cfg(all(anodized_print, anodized_panic))]
#[test]
#[should_panic(expected = r#"precondition failed:
    lower < upper"#)]
fn test_2() {
    let _ = Type::func_2(Bounds {
        lower: 42,
        upper: 1,
    });
}

#[cfg(all(anodized_print, anodized_panic))]
#[test]
#[should_panic(expected = r#"postcondition failed:
    | output | * output < upper"#)]
fn test_3() {
    let _ = Type::func_3((42, Bounds { lower: 1, upper: 5 }));
}
