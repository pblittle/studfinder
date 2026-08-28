# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- `CONTRIBUTING.md`, `CODE_OF_CONDUCT.md`, and `SECURITY.md`, plus issue forms and a
  pull request template ([#13])
- `.github/dependabot.yml`, enabling weekly cargo and GitHub Actions version updates
  rather than security updates alone ([#13])
- Package metadata in `Cargo.toml`: description, license, repository, readme,
  keywords, and categories ([#13])

### Changed

- CI: replaced the archived `actions-rs/toolchain` action with `dtolnay/rust-toolchain`,
  bumped `actions/checkout` and `actions/cache` to v4, and added `--locked` so CI fails
  when `Cargo.lock` is out of sync with `Cargo.toml` instead of silently re-resolving
  it ([#10])

### Fixed

- README listed SQLite3 as a prerequisite, but `rusqlite`'s `bundled` feature compiles
  SQLite in, so no system install is needed ([#13])

### Removed

- Unused dependencies `imageproc`, `rusttype`, `ndarray`, and `mockall`. None were
  referenced anywhere in `src/` or `tests/`. Dropping them removes 38 crates from the
  dependency tree ([#11])

### Security

- The `rand` 0.7.3 advisory listed under 0.1.0 known issues is resolved. `rand` was
  reachable only through `imageproc`, so removing `imageproc` takes `rand` and its
  `getrandom` 0.1 / `wasi` 0.9 chain out of the tree entirely, with no breaking upgrade
  needed. Removing `rusttype` also clears the unmaintained-crate warnings for `rusttype`
  and `ttf-parser` ([#11])
- `anyhow` 1.0.93 → 1.0.104 — RUSTSEC-2026-0190, unsoundness in
  `Error::downcast_mut()` when called on an error that has had context added via
  `Error::context` ([#12])

## [0.1.0] - 2026-08-28

First tagged release.

### Added

- Image-based LEGO piece identification, built on an `ImageProcessor` trait with two
  interchangeable strategies: `Scanner` (colour-based) and `Detector` (template matching)
- Colour detection with selectable standards (BrickLink or LEGO official) and
  confidence scoring based on colour purity
- Configurable scan quality: Fast, Balanced, Accurate
- Local SQLite inventory with a versioned schema
- Batch directory processing
- Inventory import/export in JSON and CSV
- CLI: `init`, `scan`, `reset`, and `inventory list | export | import`

### Fixed

- Scoped the `unwrap()` ban to production code. `.clippy.toml` documented the ban as
  production-only, but `disallowed-methods` is global, so `cargo clippy --all-targets`
  reported 67 errors — all of them in test code — keeping the `lint-all` CI job red ([#7])
- `cargo doc --warn-missing-docs` is not a valid cargo argument, so the documentation
  check failed immediately and had never run. Moved to `RUSTDOCFLAGS`, in CI and in the
  Makefile ([#7])
- README setup steps that could not succeed on a fresh clone: a placeholder clone URL,
  a `mkdir test_data` against a committed fixture, and an ImageMagick prerequisite that
  nothing uses ([#5], closes [#4])

### Security

- `imageproc` 0.23.0 → 0.23.1 — out-of-bounds read via NaN coordinates in
  bilinear/bicubic sampling, and integer overflow in the kernel size check ([#6])
- `bytes` 1.8.0 → 1.11.1 — integer overflow in `BytesMut::reserve` ([#6])
- `crossbeam-epoch` 0.9.18 → 0.9.20 — invalid pointer dereference in the `fmt::Pointer`
  impl for `Atomic` and `Shared` ([#6])

### Known issues

- RUSTSEC advisory for `rand` 0.7.3 (low, unsound with a custom logger using
  `rand::rng()`) is unresolved. `rand` is transitive via `imageproc`, which still
  requires `rand ^0.7.3`, so reaching the patched 0.8.6 needs a breaking
  `imageproc` 0.24+ upgrade.

[Unreleased]: https://github.com/pblittle/studfinder/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/pblittle/studfinder/releases/tag/v0.1.0
[#4]: https://github.com/pblittle/studfinder/issues/4
[#5]: https://github.com/pblittle/studfinder/pull/5
[#6]: https://github.com/pblittle/studfinder/pull/6
[#7]: https://github.com/pblittle/studfinder/pull/7
[#10]: https://github.com/pblittle/studfinder/pull/10
[#11]: https://github.com/pblittle/studfinder/pull/11
[#12]: https://github.com/pblittle/studfinder/pull/12
[#13]: https://github.com/pblittle/studfinder/pull/13
