# Architecture

How cardstack is put together: crate/module layout and how the pieces interact at
runtime. For *what the software must do* (behavior, per-capability), see
[`docs/specs/`](./docs/specs/); this file is the structural map, not the spec.

## Layout

A single binary crate (`cardstack`, edition 2024).

| Module | Responsibility |
|---|---|
| `main.rs` | Entry point: panic hook (terminal restore), `AlternateScreen` setup, hands off to `App`. |
| `model.rs` | Domain model — `Board`, `Task`, `Category`, `Status` — and the pure operations on them (create/delete/move/reorder). No I/O, no rendering. |
| `storage.rs` | TOML board-file (de)serialization and the XDG config-dir board location. |
| `app.rs` | Top-level `App` state (open boards, active tab) and the event/redraw loop. |
| `ui/` | ratatui rendering, built on [`ferrowl-ui`](https://github.com/TumbleOwlee/ferrowl)'s widget library (tab bar, form fields, dialogs) rather than reimplementing primitives. |

## Dependency on ferrowl-ui

`ferrowl-ui` isn't published to crates.io; it's pulled in as a git dependency
pinned to a specific rev of the [ferrowl](https://github.com/TumbleOwlee/ferrowl)
repo (see `Cargo.toml`). Bumping that rev is a deliberate action, not automatic —
do it when a needed widget/fix lands upstream, and re-verify the app still
builds and runs after.

## Runtime data flow

`main` loads every board file (or bootstraps one `Default` board, `BD-R-003`)
into `App`, then runs a poll-driven event loop: each tick draws one frame via
`ui::render`, then waits up to 100ms for a key event before redrawing. Board
mutations (once wired in later stages) autosave immediately (`ST-R-010`) rather
than going through a separate save step.
