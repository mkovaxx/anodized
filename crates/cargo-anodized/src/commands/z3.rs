use std::ffi::{CStr, CString};
use std::io::{BufRead, Write};

use z3_sys::{
    Z3_del_config, Z3_del_context, Z3_eval_smtlib2_string, Z3_mk_config, Z3_mk_context,
    Z3_set_error_handler,
};

use crate::cli::Z3Options;

type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Nul(#[from] std::ffi::c_str::NulError),
}

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

    let mut input_stream = std::io::stdin().lock();
    let mut output_stream = std::io::stdout().lock();
    while let Some(command) = parse_smtlib2_command(&mut input_stream)? {
        if command == b"(exit)" {
            break;
        }
        let command_cstr = CString::new(command)?;
        let result = unsafe { CStr::from_ptr(Z3_eval_smtlib2_string(ctx, command_cstr.as_ptr())) };
        output_stream.write_all(result.to_bytes())?;
        output_stream.flush()?;
    }

    unsafe { Z3_del_context(ctx) };

    Ok(())
}

/// Reads the next command from `input`, returning `Ok(None)` at end of input.
///
/// Assumes every line is one complete S-expression and returns it without its line ending.
fn parse_smtlib2_command(input: &mut impl BufRead) -> Result<Option<Vec<u8>>> {
    // TODO: Replace naive line-by-line input with parsing an (SMT-LIB2) S-expression.
    let mut line = Vec::new();
    if input.read_until(b'\n', &mut line)? == 0 {
        return Ok(None);
    }
    // Strip "\n" and "\r\n".
    if line.last() == Some(&b'\n') {
        line.pop();
        if line.last() == Some(&b'\r') {
            line.pop();
        }
    }
    Ok(Some(line))
}
