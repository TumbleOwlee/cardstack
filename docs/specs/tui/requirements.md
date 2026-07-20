# TUI — Requirements

Normative behavior of the terminal UI: the application shell and board-tab
model, column/card rendering, the focus and navigation model, keybindings, the
`:` command line, and the dialog mechanisms (task detail, category management,
delete confirmation).

IDs are stable and append-only (`UI-R-nnn`). See [`../README.md`](../README.md).

Companion document: [`api-contract.md`](./api-contract.md) — the exhaustive `:`
command list and keybinding table.

---

## Shell & board tabs

**UI-R-001** — The application shall present a full-screen terminal UI in the
alternate screen buffer with raw mode enabled, and shall restore the terminal on
normal exit, error exit, and from a panic hook.

**UI-R-002** — The screen shall be laid out top-to-bottom as: a one-row board
tab bar, the three-column board area, and a one-row command line. The board
area absorbs the remaining height.

**UI-R-003** — Every board loaded at startup (per `ST-R-020`) shall appear as a
tab in the tab bar; the application shall track one active (displayed) board at
a time.

**UI-R-004** — Each tab's title shall be prefixed with its board's index, e.g.
`[0] Default`. `Ctrl+T` followed by a digit switches directly to the board at
that index (a no-op if no board has that index); any other key cancels the
pending switch without effect.

## Column & card layout

**UI-R-010** — The board area shall show exactly three side-by-side columns,
left to right: `Open`, `InProgress`, `Done`, each independently scrollable and
each showing only the active board's tasks with that status, in that status's
manual order (`BD-R-030`). When a column's cards don't all fit in its area, the
column scrolls to keep the focused card fully visible (`UI-R-022`); cards above
the scroll position are simply not drawn, not truncated. Column titles are
centered.

**UI-R-011** — Each task shall render as a multi-line bordered card with a
1-cell horizontal margin around its content: a labels row first (`UI-R-014`)
if the task has any labels, followed by a blank row (only when the labels row
is present), the title in bold below that, the description (non-bold, wrapped
over multiple rows) below that if the description is non-empty (an empty
description renders no row and adds no height), then a footer row with the
category name at the bottom-left and the due date at the bottom-right,
present only if the task has a category or a due date (a task with neither
renders no footer row and no gap before it); when the footer row is present,
a blank row separates it from whatever is above (the description if present,
otherwise the title). The category name renders in uppercase as a bold badge
with its category color as the background and black or white foreground text,
whichever has higher contrast against that background. A card's height is not
fixed: it grows to fit however many rows its description and labels wrap to
at the column's width, so no description or label text is ever clipped.

**UI-R-012** — A card's border and title shall be colorized using its task's
category color (`BD-R-040`); a task with no category renders in white
(`BD-R-044`).

**UI-R-013** — A card whose due date is in the past and whose status is not
`Done` shall render its due-date text in the theme's error color. A due date
that is today (regardless of status) renders in yellow. Every other due date
renders in the default text color.

**UI-R-014** — A task with one or more labels (`BD-R-010`) renders a labels
row as the card's first row, above the title: each label as an uppercase badge
(rendered ` LABEL `, a 1-cell space margin either side of the text, no
brackets — same shape as `UI-R-011`'s category badge), space-separated from
its neighbors, with a background color and black-or-white foreground text
chosen by the same contrast rule as `UI-R-011`'s category badge. Badge colors
are assigned by each label's first-seen position among the active board's
tasks (in board task order), indexed into the same fixed palette `BD-R-041`
uses for categories but walked in reverse order, cycling if there are more
distinct labels than palette entries; this mapping is recomputed on every
render and is not persisted. The
labels row wraps to as many rows as needed, using the same greedy word-wrap
as the description — no label is ever hidden. A task with no labels renders
no labels row and adds no height.

## Focus & navigation

**UI-R-020** — Exactly one column, and within it exactly one card (if the
column is non-empty), is focused at a time.

**UI-R-023** — The focused card shall be visually distinguished from other
cards (e.g. a distinct border style plus emphasis), while still showing its
category color. The focused column's own border shall also always be
visually distinguished (whether or not it has a focused card), so focus is
never invisible.

**UI-R-021** — `h`/`←` and `l`/`→` move column focus left/right; at the
leftmost (`Open`) or rightmost (`Done`) column they are a no-op.

**UI-R-022** — `j`/`↓` and `k`/`↑` move card focus down/up within the focused
column; at the bottom or top card they are a no-op. Moving column focus (UI-R-021)
into a column keeps a sensible focused card (e.g. the topmost, or the last
focused card in that column if still present).

## Status move & reorder

**UI-R-030** — `H`/`shift+←` and `L`/`shift+→` on the focused card move its
status one column left/right (`BD-R-020`); the moved card remains focused,
following it into the target column.

**UI-R-031** — `J`/`shift+↓` and `K`/`shift+↑` on the focused card reorder it
down/up within its column (`BD-R-031`); the moved card remains focused.

## Dialogs

**UI-R-040** — `Enter` on a focused card opens the task detail dialog
pre-filled with that task's fields; editing and confirming updates the task in
place (autosaved per `ST-R-010`).

**UI-R-041** — The task detail dialog shall present title, due date, category
(selectable from the board's categories), and labels as discrete fields, plus a
multi-line textarea for the description, all editable in one dialog. The
dialog's content has a 1-cell vertical and 2-cell horizontal margin. Due date
and category share one row, side by side; the category selection field is
always 1 row tall (plus its border). The description textarea is 4 rows tall
(plus its border). The `[ Save ]` action is right-aligned on its row.

**UI-R-042** — `d` on a focused card opens a yes/no confirmation dialog;
confirming deletes the task (`BD-R-013`), declining or cancelling leaves it
unchanged. No direct-delete path bypasses this confirmation. The dialog's
content has a 2-cell horizontal and 1-cell vertical margin. It shows the
name of the task or board to be deleted in a non-focusable, non-editable
input field titled "Board/Task name". Below it, `[ Yes ]` and `[ No ]` sit
together, right-aligned on the same row, `[ Yes ]` to the left of `[ No ]`.
`[ No ]` is focused by default when the dialog opens.

**UI-R-043** — The category-management dialog (opened via `:categories`) shall
have a centered dialog title reading "Modify Category". It lists the active
board's categories, rendered in uppercase, in a titled ("Category") scrollable
selection list that always fills its allotted area (its border does not
shrink to the item count), with a 2-cell horizontal and 1-cell vertical
margin around the dialog's content. Below the list, an always-visible input
field adds a new category on `Enter`, followed by a centered, white, static
line of text describing its keybindings. `Tab`/`Shift+Tab` switches focus
between the list and that input field. `Enter` on a selected item switches
the input field into rename mode, prefilled with that item's name, and moves
focus to it; confirming renames the selected category instead of adding one.
Recoloring and deleting act on the selected item directly (no separate
input). The selected item's highlight uses that category's own color as the
background, with black or white foreground text (`UI-R-011`'s contrast rule)
instead of the theme's generic selection color.

**UI-R-044** — Any open dialog shall close on `Esc` without applying unsaved
edits made in that dialog (a confirmed edit takes effect only on the dialog's
explicit confirm action).

## Command line

**UI-R-050** — `:` shall enter command-line mode, showing a one-row input at
the bottom of the screen with a visible text cursor, same as any other text
input field; `Enter` submits the typed command, `Esc` cancels without effect.
When command-line mode is not active and no error is displayed (`UI-R-051`),
the command line shows the static hint text `: command`, in white, in its
place.

**UI-R-051** — The generic commands are `:new-task`, `:new-board <name>`,
`:rename <name>`, `:swap <i> <j>`, `:delete`, and `:categories` (see
[`api-contract.md`](./api-contract.md) for exact syntax). An unrecognized
command shall display a single-line error message in the command line,
styled as an error, and have no other effect. The application shall never
write to stdout or stderr while the terminal UI is active.

**UI-R-052** — `:new-task` shall open the task detail dialog blank
(`BD-R-012` for its default status), equivalent to the create-flow of
`UI-R-040`.

**UI-R-053** — `:new-board <name>` shall create a new, empty board named
`<name>` (`ST-R-011`), open it as a new tab, and switch focus to it.

**UI-R-057** — `:rename <name>` shall rename the active board to `<name>`
(`ST-R-012`); a blank `<name>` is a no-op.

**UI-R-058** — `:swap <i> <j>` shall swap the tab positions of the boards at
indices `<i>` and `<j>` (`UI-R-004`); either index out of range is a no-op.
The active board follows its own tab if it was one of the two swapped, so the
same board stays focused.

**UI-R-059** — `:delete` shall open a yes/no confirmation dialog (in the same
style as `UI-R-042`) naming the active board; confirming deletes it
(`ST-R-014`) and switches focus to another open tab, declining or `Esc`
leaves it unchanged. No direct-delete path bypasses this confirmation.

**UI-R-054** — While command-line mode (`UI-R-050`) is active, a popup shall
list the available commands (`UI-R-051`) with their syntax and effect; it
shows continuously during command-line entry and closes automatically when
command-line mode ends.

**UI-R-055** — Every rendered area — the full screen and every dialog/popup
interior — shall be filled with the theme's default background color before
the widgets it contains are drawn, so no cell shows the terminal's own
background.

**UI-R-056** — Every bordered/titled element (columns, dialogs, input fields,
selection/list widgets) that is not currently focused shall render its border
and title in white. This does not apply to a card's border, which is always
colorized by its category (`UI-R-012`) regardless of focus.
