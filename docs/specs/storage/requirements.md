# Storage — Requirements

Normative behavior of persistence: the save-file format, location, save/load
semantics, and autosave.

IDs are stable and append-only (`ST-R-nnn`). See [`../README.md`](../README.md).

---

**ST-R-001** — Each board shall be persisted as its own TOML file.

**ST-R-002** — Board files shall live under the platform config directory
(XDG on Linux, resolved via a `ProjectDirs`-style lookup), in a `boards`
subdirectory — never the current working directory.

**ST-R-003** — A board's filename shall be derived from its name, sanitized to
a filesystem-safe form; the board's display name is stored inside the file as
the authoritative name (the filename is a lookup key, not the source of truth).

**ST-R-010** — Every mutation to a board (task create/edit/delete/move/reorder,
category create/edit/delete, board rename) shall be written to that board's
file immediately (autosave) — there is no explicit save command and no
unsaved-state to lose on exit or crash.

**ST-R-011** — Creating a new board shall write its file immediately, even
before any task is added, so an empty board persists across restarts.

**ST-R-012** — Renaming a board shall write its file under the new name's
sanitized filename (`ST-R-003`) and remove the old filename's file, so no
stale duplicate remains on disk.

**ST-R-020** — On startup, every board file found in the boards directory
shall be loaded and opened as a tab. A board file that fails to parse shall be
skipped (not loaded, not deleted) with a warning logged, rather than aborting
startup — a single corrupt file shall not block access to the user's other
boards.

**ST-R-021** — A save immediately followed by a load of the same board shall
reproduce identical board state: task fields, status, manual order within each
status, and category definitions.

**ST-R-014** — Deleting a board shall remove its file from the boards
directory immediately, and remove it from the persisted tab order
(`ST-R-022`). Deleting the last remaining board leaves no board file behind;
the next startup's empty-state bootstrap (`ST-R-020`, a fresh `Default`
board) applies immediately rather than leaving the application with zero
boards open.

**ST-R-022** — The order in which boards were opened as tabs (`UI-R-003`,
`UI-R-057`) shall be persisted in the config directory and restored exactly on
the next startup, independent of filesystem directory-listing order. Any
mutation that changes tab order (`UI-R-053` new board, `UI-R-057` rename,
`UI-R-058` swap) shall write the new order immediately.

**ST-R-013** — On startup, before loading or writing any board file, the
application shall acquire an exclusive lock on the config directory (a lock
file). If the lock is already held by another running instance, the
application shall exit immediately with an error message, printed before the
alternate screen is entered, rather than risk two processes writing the same
board files concurrently. The lock is released on normal exit, error exit,
and panic.
