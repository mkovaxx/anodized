use crate::cli::Z3Options;

type Result<T> = std::result::Result<T, Error>;

pub fn emulate_smt2_in(_: Z3Options) -> Result<()> {
    todo!()
}

#[derive(Debug, thiserror::Error)]
pub enum Error {}
