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
