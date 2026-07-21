# TUI — Edge Cases

Behavior that is deliberately simple rather than fully validated. See
[`requirements.md`](./requirements.md) for the normative "shall" statements
these refine.

---

**Malformed due-date text.** The task detail dialog's due date field
(`UI-R-041`) is free text, not a date picker. If it doesn't parse as
`YYYY-MM-DD` when the dialog is confirmed, the task's due date is silently
cleared (treated as "no due date") rather than rejecting the confirm or
surfacing a validation error. There is no separate validation UI in the MVP.

**Empty title.** `BD-R-010` requires a non-empty title. Confirming the task
detail dialog with a blank (or whitespace-only) title is a no-op — the
dialog stays open rather than saving an invalid task or silently discarding
the edit.

**Leaving the category dialog's rename input uncommitted.** `Tab`/`Shift+Tab`
away from the add/rename input (`UI-R-043`) discards whatever was typed and
resets the field to empty add-mode, rather than preserving a half-finished
rename for later. `Esc` still closes the whole category dialog outright
(`UI-R-044`), same as it discards a pending add.

**Label colors survive reordering and deletion, reset on reload.** Once a
label has been assigned a color (`UI-R-014`), that assignment is cached for
the lifetime of the running board, so reordering, moving, or deleting tasks
never shifts another label's color. The cache lives only in memory:
restarting the app (or reloading the save file) recomputes first-seen order
from scratch against the freshly loaded task order, which need not match the
order the previous session settled on. Accepted: no persisted label→color
table exists in the MVP.

**Filter does not affect label colors.** An active filter (`UI-R-060`) hides
cards from the view but label badge colors (`UI-R-014`) are still computed over
**all** of the board's tasks, not just the visible ones. A card's label colors
therefore do not shift when a filter is applied or cleared. Accepted: filtering
is a pure view concern and must not perturb the label→color mapping.

**Filter hiding the focused card.** When applying a filter (`UI-R-060`) hides
the currently focused card, focus resyncs to the first still-visible card —
preferring the focused column, then falling back so focus lands on some visible
card if one exists. If the filter empties the focused column entirely, that
column has no focused card (`UI-R-020`), consistent with an empty column.

**Filter values cannot contain whitespace.** `:filter` terms are
whitespace-separated (`UI-R-060`), so a category or label whose name contains a
space cannot be named in full — `:filter category=Project X` reads `Project` as
the category value and `X` as an unknown term (an error). Accepted for the MVP:
the command surface stays a simple space-delimited term list rather than
introducing quoting.

**Filter matching nothing.** A filter whose condition matches no task renders
the columns empty rather than surfacing an error or refusing the filter — the
same as a board with no matching tasks. Clearing the filter restores the cards.
