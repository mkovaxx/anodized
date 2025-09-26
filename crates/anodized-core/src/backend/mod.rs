pub mod function;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Backend {
    pub disable_runtime_checks: bool,
}

impl Backend {
    pub const DEFAULT: Backend = Backend {
        disable_runtime_checks: false,
    };

    pub const NO_CHECKS: Backend = Backend {
        disable_runtime_checks: true,
    };
}
