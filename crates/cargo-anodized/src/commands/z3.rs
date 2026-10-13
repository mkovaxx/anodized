use std::ffi::{CStr, CString};
use std::io::{BufRead, Write};

use z3_sys::{
    Z3_del_config, Z3_del_context, Z3_eval_smtlib2_string, Z3_mk_config, Z3_mk_context,
    Z3_set_error_handler,
};

use crate::cli::Z3Options;

type Result<T> = std::result::Result<T, Error>;

pub fn emulate_smt2_in(_: Z3Options) -> Result<()> {
    let ctx = unsafe {
        let cfg = Z3_mk_config().expect("Z3_mk_config failed");
        let ctx = Z3_mk_context(cfg).expect("Z3_mk_context failed");
        Z3_del_config(cfg);
        // Z3's default error handler prints "Error: ..." and calls exit(1) on the
        // first failing command. With no handler, the error text is simply part of
        // the string Z3_eval_smtlib2_string returns, as with the z3 CLI.
        Z3_set_error_handler(ctx, None);
        ctx
    };

    let mut out = std::io::stdout().lock();
    for line in std::io::stdin().lock().lines() {
        // TODO: Replace naive line-by-line input with parsing an (SMT-LIB2) S-expression.
        let line = line.unwrap();
        if line == "(exit)" {
            break;
        }
        let c_line = CString::new(line).unwrap();
        let res = unsafe { CStr::from_ptr(Z3_eval_smtlib2_string(ctx, c_line.as_ptr())) };
        out.write_all(res.to_bytes()).unwrap();
        out.flush().unwrap();
    }

    unsafe { Z3_del_context(ctx) };

    Ok(())
}

#[derive(Debug, thiserror::Error)]
pub enum Error {}
