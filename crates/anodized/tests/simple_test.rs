#![feature(contracts)]

use core::contracts::*;

#[requires(true)]
#[requires(true)] // This line causes an error.
#[ensures(|_| true)]
#[ensures(|_| true)]
fn some_function() {}
