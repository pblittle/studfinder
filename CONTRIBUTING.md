# Contributing

Thanks for taking an interest in studfinder.

## Getting set up

```bash
git clone https://github.com/pblittle/studfinder.git
cd studfinder
make build
```

A stable Rust toolchain is all you need. SQLite is compiled in through
`rusqlite`'s `bundled` feature, so there is no system database to install.

## Before opening a pull request

Run what CI runs:

```bash
make lint-all   # clippy, rustfmt, clippy across all targets, then the doc build
make test       # the test suite
make format     # apply rustfmt
```

`make check` runs all of the above plus `cargo audit`, which needs
`cargo install --locked cargo-audit` first. CI runs the audit separately, so you
do not have to install it locally.

Two things are easy to trip over:

- CI builds with `cargo build --locked`. If you add, remove, or bump a
  dependency, commit the updated `Cargo.lock` alongside `Cargo.toml`.
- `unwrap()` is disallowed in production code through `.clippy.toml`. Return a
  `Result` and let the caller decide. Tests are exempt, since a panic there is
  the failure signal.

## Commit messages

This project follows [Conventional Commits](https://www.conventionalcommits.org/):
`feat:`, `fix:`, `docs:`, `chore:`, `ci:`, `refactor:`, `test:`. The scope is
optional, so `fix(scanner): ...` and `fix: ...` are both fine.

## Reporting a bug

Open an issue with the command you ran, what you expected, and what actually
happened. If it involves a specific image, describing its size and dominant
color usually matters more than the file itself.

For anything security-related, see [SECURITY.md](SECURITY.md) instead.
