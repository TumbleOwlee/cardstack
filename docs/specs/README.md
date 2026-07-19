# Cardstack Specs

Authoritative specification of cardstack's behavior, split by capability area.

These files are **normative**: the code is expected to conform to them, not the
other way around. When code and spec disagree, that is a defect in one of them —
resolve it, don't paper over it.

## Areas

| Area | Prefix | Covers |
|---|---|---|
| [`board/`](./board/) | `BD-R-nnn` | Domain model: boards, columns, cards, card operations |
| [`storage/`](./storage/) | `ST-R-nnn` | Save-file format, save/load, migration |
| [`tui/`](./tui/) | `UI-R-nnn` | Screen layout, keybindings, commands, navigation |

Cross-cutting: [`non-functional-requirements.md`](./non-functional-requirements.md).

## Rules for writing specs

**1. No code pointers.** Never cite `file:line`, function names, struct names, or
crate-internal identifiers. A spec states *what must be true*, not where it is
implemented — code pointers rot on every refactor and turn the authoritative doc
into a liar. Public, user-facing names (config keys, keybindings, CLI flags) are
part of the contract and *are* spec content.

**2. Requirement IDs are stable and append-only.** Each requirement carries an ID,
prefixed per area (e.g. `BD-R-nnn` for `board/`, `ST-R-nnn` for `storage/`,
`UI-R-nnn` for `tui/`, `NF-R-nnn` for non-functional). Assign a prefix when an
area is created and record it in the table above. Never renumber. Never reuse a
retired ID. Reference requirements by ID in commits, PRs, and agent instructions.

**3. Owner is the behavior, not the surface.** Give each requirement to the area
that owns the behavior it controls, so one change touches one file.

**4. Requirements are testable.** Write "shall" statements with observable
outcomes. "Deleting a card shall remove it from its column and re-index the
remaining cards" is a requirement. "Card deletion works correctly" is not.

**5. Known gaps are specified, not hidden.** Behavior that is ugly but
intentional belongs in the area's `edge-cases.md` as a stated constraint — so it
is not mistaken for an oversight and silently "fixed".

## Per-area files

Not every area needs every file; add and drop based on need.

| File | Contains |
|---|---|
| `requirements.md` | Numbered, testable "shall" statements. Every area has one. |
| `api-contract.md` | The area's stable public surface: keybindings, commands, CLI flags. |
| `data-contract.md` | Wire and file formats: save-file schema, payload shapes. |
| `edge-cases.md` | Boundary behavior, error semantics, and stated known limitations. |

## Keeping specs true

Before changing code in an area, read that area's `requirements.md`. If the
change contradicts the spec, update the spec **in the same commit**. A behavior
change with no spec change is an incomplete change.
