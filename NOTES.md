We want to completely revamp Anodized's main README, along the following lines.

Main Title: Anodized Annotations

High-Level Structure

1. What Is Anodized?

- Helps document properties of your code with precision and confidence.

- Table comparing Anodized `spec` annotations to comments, assertions, and types.
  Integrated with the code? Y/N
  Centralized and grokkable? Y/N
  Write arbitrary properties? Y/N
  Comes with a formal grammar? Y/N
  Works with stable Rust? Y/N
  Automated runtime checks? Y/N
  Support static analysis? Y/N

2. How Is Anodized Different?

Rust has a rich ecosystem of static analysis tools (verifiers, refinement types, etc.), fuzzers/property-based, and so on.

The main shortcoming of those tools are as follows:

- Most of them extend Rust in non-trivial ways, making then largely incompatible, and more effort to learn.
- Most of them do not work with stable Rust.
- Most of them do not integrate with other tools.

- Table comparing Anodized to other tools, such as `contracts`, Flux, Verus, Prusti, Kani, Aeneas, Creusot, etc.

3. Roadmap

- Table showing existing and planned runtime behaviors:
  current: check-and-panic, check-and-print, no-check
  planned: check-and-log, check-and-trace, check-and-trap (breakpoint)

- Table showing what Rust entities the `spec` annotaion supports
  current: stand-alone `fn`, method `fn`
  planned: `struct`/`enum`, `trait` methods,
