# Anodized Example: Binary Search

All tests in this example, including the fuzz target, must be compiled with
Anodized's `print`, `panic`, and `try` configurations enabled. Set them through
`RUSTFLAGS` when running a test command.

## Property-Based Testing

Run the `proptest` test suite:

```sh
RUSTFLAGS="--cfg anodized_print --cfg anodized_panic --cfg anodized_try" cargo test --test proptest
```

Run the `quickcheck` test suite:

```sh
RUSTFLAGS="--cfg anodized_print --cfg anodized_panic --cfg anodized_try" cargo test --test quickcheck
```

## Fuzzing

Run the fuzzing target for `cargo-fuzz` via the following command:

```sh
RUSTFLAGS="--cfg anodized_print --cfg anodized_panic --cfg anodized_try" \
  cargo +nightly fuzz run fuzz_target_1 -Ztarget-applies-to-host -Zhost-config
```
