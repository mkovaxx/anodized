#![cfg_attr(anodized_charon, feature(register_tool))]
#![cfg_attr(anodized_charon, register_tool(charon))]
#![allow(clippy::unit_cmp, clippy::needless_return)]

use anodized::spec;
use std::cell::RefCell;

struct ExecLog(RefCell<Vec<&'static str>>);

#[allow(unused)]
impl ExecLog {
    fn new() -> Self {
        Self(RefCell::new(Vec::new()))
    }

    fn push(&self, entry: &'static str) {
        self.0.borrow_mut().push(entry);
    }

    fn into_vec(self) -> Vec<&'static str> {
        self.0.into_inner()
    }
}

#[spec(
    requires: [
        return log.push("requires1") == (),
        return log.push("requires2") == (),
    ],
    maintains: [
        return log.push("maintains1") == (),
        return log.push("maintains2") == (),
    ],
    captures: [
        _alias1 = return log.push("captures1"),
        _alias2 = return log.push("captures2"),
    ],
    ensures: [
        return log.push("ensures1") == (),
        return log.push("ensures2") == (),
    ],
)]
#[allow(unused)]
fn func(log: &ExecLog) {
    log.push("body");
    return;
}

#[cfg(any(anodized_panic, anodized_print))]
#[test]
fn execution_order() {
    let log = ExecLog::new();
    func(&log);

    // Verify the exact execution order
    assert_eq!(
        log.into_vec(),
        [
            "requires1",
            "requires2",
            "maintains1",
            "maintains2",
            "captures1",
            "captures2",
            "body",
            "maintains1",
            "maintains2",
            "ensures1",
            "ensures2",
        ]
    );
}

#[spec(
    requires: [
        log.push("requires1") != (),
        log.push("requires2") != (),
    ],
    maintains: [
        log.push("maintains1") != (),
        log.push("maintains2") != (),
    ],
    captures: [
        _alias1 = log.push("captures1"),
        _alias2 = log.push("captures2"),
    ],
    ensures: [
        log.push("ensures1") != (),
        log.push("ensures2") != (),
    ],
)]
#[allow(unused)]
fn func_all_conditions_fail(log: &ExecLog) {
    log.push("body");
    return;
}

#[cfg(all(anodized_print, not(anodized_panic)))]
#[test]
fn execution_order_print_only() {
    let log = ExecLog::new();
    func_all_conditions_fail(&log);

    // Verify the exact execution order
    assert_eq!(
        log.into_vec(),
        [
            "requires1",
            "requires2",
            "maintains1",
            "maintains2",
            "captures1",
            "captures2",
            "body",
            "maintains1",
            "maintains2",
            "ensures1",
            "ensures2",
        ]
    );
}

#[spec(
    requires: [
        log.push("requires1") == (),
        log.push("requires2") == (),
    ],
    maintains: [
        log.push("maintains1") == (),
        log.push("maintains2") == (),
    ],
    captures: [
        _alias1 = log.push("captures1"),
        _alias2 = log.push("captures2"),
    ],
    ensures: [
        log.push("ensures1") == (),
        log.push("ensures2") == (),
    ],
)]
#[allow(unused)]
async fn async_func(log: &ExecLog) {
    log.push("body");
    return;
}

#[cfg(any(anodized_panic, anodized_print))]
#[test]
fn async_execution_order() {
    let log = ExecLog::new();
    pollster::block_on(async_func(&log));

    assert_eq!(
        log.into_vec(),
        [
            "requires1",
            "requires2",
            "maintains1",
            "maintains2",
            "captures1",
            "captures2",
            "body",
            "maintains1",
            "maintains2",
            "ensures1",
            "ensures2",
        ]
    );
}

#[spec(maintains: { self.log.push(self.label); true })]
pub struct TypeWithSpec<'a> {
    label: &'static str,
    log: &'a ExecLog,
}

#[spec(
    requires: { i2.log.push("pre"); true },
    ensures: { i2.log.push("post"); true },
)]
pub fn func_io<'a>(
    i1: Spec!(TypeWithSpec<'a>),
    i2: Spec!(&mut TypeWithSpec, inout),
    _: Spec!(&TypeWithSpec),
) -> Spec!(TypeWithSpec<'a>) {
    TypeWithSpec {
        label: "out",
        log: i1.log,
    }
}

#[cfg(anodized_panic)]
#[test]
fn type_spec_check_execution_order() {
    let log = ExecLog::new();

    let i1 = TypeWithSpec {
        label: "in1",
        log: &log,
    };
    let mut i2 = TypeWithSpec {
        label: "in2",
        log: &log,
    };
    let i3 = TypeWithSpec {
        label: "in3",
        log: &log,
    };
    let _ = func_io(i1, &mut i2, &i3);

    assert_eq!(
        log.into_vec(),
        ["in1", "in2", "in3", "pre", "out", "in2", "post"]
    );
}
