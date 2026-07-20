# Board — Requirements

Normative behavior of the board domain model: boards, tasks, categories, labels,
and the operations that create, move, reorder, edit, and remove them.

IDs are stable and append-only (`BD-R-nnn`). See [`../README.md`](../README.md).

---

## Boards

**BD-R-001** — A board shall have a name (unique among a user's open boards) and
an ordered list of tasks.

**BD-R-002** — A board shall have zero or more user-defined categories.

**BD-R-003** — If no boards exist at startup (per `ST-R-020`), the application
shall create one empty board named `Default` and open it, rather than starting
with no boards and no active tab.

## Tasks

**BD-R-010** — A task shall have: a title (required, non-empty), a description
(optional free text), a due date (optional, a calendar date with no time
component), a category (optional, references one of the board's categories),
zero or more labels (free-form strings), and a status.

**BD-R-011** — A task's status shall be exactly one of `Open`, `InProgress`, or
`Done`.

**BD-R-012** — Creating a task shall default its status to the column it was
created from (the currently focused column). If no column is focused when a
task is created (e.g. via `:new-task` with no prior focus), its status shall
default to `Open`.

**BD-R-013** — Deleting a task shall remove it from its board's task list.

**BD-R-014** — All of a task's fields (title, description, due date, category,
labels, status) shall be editable after creation.

## Status changes

**BD-R-020** — Columns have a fixed left-to-right order: `Open`, `InProgress`,
`Done`. Moving a task's status "right" advances it one column in that order;
moving it "left" reverses one column. A move in a direction with no neighboring
column (right from `Done`, left from `Open`) is a no-op.

**BD-R-021** — When a task's status changes, it shall be placed after every
other task currently in the target status (appended to the bottom of the
target column), never inserted in the middle.

## Ordering within a column

**BD-R-030** — Within a single status, tasks have a manual, user-controlled
order. A newly created task is appended to the bottom of its column.

**BD-R-031** — Reordering a task moves it one position up or down among tasks
that share its status only; tasks of other statuses are not affected and do not
count as neighbors. Reordering past the top or bottom of the same-status group
is a no-op.

## Categories

**BD-R-040** — A category has a name (unique within its board) and a color.

**BD-R-041** — A newly created category's color shall be auto-assigned from a
fixed palette; the assignment need not be user-visible before creation.

**BD-R-042** — Renaming or recoloring a category shall update how every task
referencing it is displayed; it does not change task data other than through
that reference.

**BD-R-043** — Deleting a category shall clear the category field (not delete)
on every task that referenced it — a task is never removed as a side effect of
category deletion.

**BD-R-044** — A task with no category set shall render its card border and
title in white, not an error state.
