# Anodized Example: Binary Search

All tests in this example, including the fuzz target, are compiled with
Anodized's `print`, `panic`, and `try` configurations enabled by
[`.cargo/config.toml`](.cargo/config.toml).

## Property-Based Testing

Run the `proptest` test suite:

```sh
cargo test --test proptest
```

Run the `quickcheck` test suite:

```sh
cargo test --test quickcheck
```

## Fuzzing

Run the fuzzing target for `cargo-fuzz` via the following command:

```sh
cargo +nightly fuzz run fuzz_target_1 -Ztarget-applies-to-host -Zhost-config
```
