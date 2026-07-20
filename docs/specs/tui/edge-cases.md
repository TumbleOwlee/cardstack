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

**Label color reassignment on deletion.** Label badge colors (`UI-R-014`) are
derived live from first-seen order across the active board's tasks, not
stored. Deleting or editing away the only task using an earlier-occurring
label shifts the first-seen index of every label after it, which can change
their rendered color within the same session. Accepted: no persisted
label→color table exists in the MVP.
