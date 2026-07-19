# Contributing to Cardstack

## Setup

Written in Rust (stable toolchain). Install via [rustup.rs](https://rustup.rs/), then:

```sh
git clone <your-fork>
cd cardstack
cargo build
```

## Project Layout

See [`ARCHITECTURE.md`](./ARCHITECTURE.md) for structure and [`PRD.md`](./PRD.md)
for product framing.

Cardstack is **spec-driven**: [`docs/specs/`](./docs/specs/) is the authoritative
specification of what the software must do, split by capability area. The code is
expected to conform to it. Before changing behavior, read the relevant area's
`requirements.md`.

## Before Submitting

```sh
cargo fmt --check
cargo clippy -- -D warnings
cargo check
cargo test
```

## Pull Requests

- Branch off `main` and open your PR against `main`.
- Keep PRs focused — one feature or fix per PR.
- Add or update tests for behavior changes.
- **Update the spec in the same PR.** When you change behavior, update the
  relevant `docs/specs/<area>/` file(s) — they are the authoritative source, not
  a one-time snapshot. New requirements get a fresh, appended ID (never renumber
  or reuse). A behavior change with no spec change is incomplete.

## Reporting Issues

Open a GitHub issue with steps to reproduce and your platform.
