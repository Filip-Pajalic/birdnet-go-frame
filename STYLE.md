# Code style

All Rust in this repository follows [Google's Rust Style Guide](https://google.github.io/styleguide/rustguide.html),
and through it the official Rust style guide and API Guidelines. Tooling enforces what it can;
review covers the rest.

## Enforced by tooling

- **Formatting:** `cargo +nightly fmt` with the checked-in `rustfmt.toml` (`max_width = 100`,
  `hex_literal_case = "Lower"`, `format_macro_matchers = true`, `imports_granularity = "Module"`).
  These options need nightly rustfmt; the code builds on stable. `#[rustfmt::skip]` only for
  deliberate alignment or generated code.
- **Lints:** the `[workspace.lints]` table in `Cargo.toml` denies the lints the guide names
  (`borrowed_box`, `ptr_arg`, `should_implement_trait`, `wrong_self_convention`,
  `new_without_default`, `len_without_is_empty`), glob imports, and `unwrap()`; it warns on
  `expect()`, missing docs and undocumented panics, and forbids `unsafe` and unstable features.
- CI runs `fmt --check`, `clippy -D warnings` and the tests on every push.

## Review checklist

- [ ] One `use` per module; types and traits imported and used unqualified; top-level functions
      called through their module (`e6::encode`); traits imported in the function that uses them
      unless used throughout the file. No preludes, no globs except `use super::*` in tests.
- [ ] Small crates, one file per abstraction, file named after its main item. `lib.rs` and other
      non-leaf modules hold only `mod` declarations and re-exports.
- [ ] Panics only for bugs; `expect("why")` with an `#[expect(clippy::expect_used, reason = ...)]`
      where an invariant makes failure impossible. `Result` for I/O and network failures,
      `Option` for simple absence. Error types implement `Display` and `Error`, are `#[must_use]`,
      and are `Copy` where they can be.
- [ ] `if let` / `match` / `let ... else` instead of `is_some()` followed by `unwrap()`.
- [ ] Explicit types on `let` for `.parse()`, `.into()`, `collect()` and other non-obvious results.
      Short iterator chains; no complex chain in a `for` header; no `for_each`.
- [ ] Enums instead of unclear `bool` parameters; newtypes instead of aliases for distinct
      meanings; named fields instead of tuples outside very local code.
- [ ] Borrowing and unique ownership over `Rc`/`Arc` and `.clone()`; `Arc` only to share across
      threads.
- [ ] Traits only for generic programming; generics (`T: Trait`) over `dyn Trait`.
- [ ] `#[derive]` where possible; standard traits (`FromStr`, `Display`, `Default`) instead of
      look-alike methods; `len()` comes with `is_empty()`.
- [ ] Public items documented with `///` (crates and modules with `//!`) in full, punctuated
      sentences; `# Errors` and `# Panics` sections where they apply; an example somewhere in
      every library crate. No comments that restate the code.
- [ ] Never read the contents of a poisoned mutex.
- [ ] No code before `main()`, no proc macros of our own, `macro_rules!` only when plain Rust
      cannot do it.
- [ ] `unsafe` only where no safe alternative exists, isolated in one module, with a `// SAFETY:`
      comment and a justification. Library crates that need it opt out of the workspace's
      `unsafe_code = "forbid"` explicitly.

## Where we differ from the guide

The guide's ban on the default `#[test]` harness applies to Chromium, Android and Fuchsia, which
have their own; this is a Cargo project, so tests use `#[test]`. Tests may panic on failed
assumptions, and test files say so with `#![allow(clippy::expect_used)]`.
