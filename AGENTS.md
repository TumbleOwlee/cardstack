# AGENTS.md

Router for AI coding agents working in this repo. Read this first; it points to
everything else.

## What this repo is

Cardstack — a Rust TUI kanban board. A single binary crate today (`src/main.rs`
only); see [`ARCHITECTURE.md`](./ARCHITECTURE.md) for current structure and
[`PRD.md`](./PRD.md) for product framing. Both are early-stage placeholders —
fill them in as the product takes shape rather than treating them as fixed.

## Spec-driven — read this before you change behavior

`docs/specs/` is the **authoritative** specification: the code is expected to
conform to it, not the other way around. Before you edit code in an area, read
that area's `requirements.md`. A behavior change with no spec change is
incomplete — the workflow below is how the two stay together.

**`main` never contains an unfinished spec.** A requirement on `main` is a
statement about code that exists and is tested. A feature branch may hold a spec
commit ahead of its implementation (see the workflow); `main` may not, which the
squash merge guarantees.

If the code and the spec already disagree and it is *not* what you were asked to
fix: **stop and raise it as its own task.** Do not fold the fix into the change
in flight — it silently widens work that was already approved, and the fix
deserves its own review.

Specs contain no `file:line` pointers by design — locate code with your own
search tools. Requirements have stable IDs (`BD-R-*` board, `ST-R-*` storage,
`UI-R-*` tui, `NF-R-*` non-functional — see
[`docs/specs/README.md`](./docs/specs/README.md)); reference them in commits and
PRs.

## Workflow — follow this for every behavior change

This project's workflow **replaces** any generic workflow skill (including
`/workflow`); do not run one here. `docs/specs/` already serves as the PRD and
the design record — a second design-artifact system would only give the "why"
two homes to diverge in.

**It triggers on behavior change, not on size.** Ask: *does this change what the
software is required to do?* If yes — a new feature, a changed keybinding,
different observable semantics — the full workflow applies, however small the
diff. If no — a refactor, a rename, perf work with identical semantics, tests,
docs — there is no spec diff to approve, so skip the gates and just do the work.
Size decides how many *stages* the plan has, never whether the gates exist.

Work on a branch off `main`, never on `main` itself. `<type>/<slug>`,
conventional-commit type (`feat/`, `fix/`, `docs/`).

1. **Read the affected area's spec.** Use the routing table below to find it.
   Read `requirements.md` and `edge-cases.md` before proposing anything —
   `edge-cases.md` records behavior that is ugly *on purpose*. If the change
   doesn't fit any existing area, this change is likely the one creating a new
   one — add it to [`docs/specs/README.md`](./docs/specs/README.md)'s area table
   with a fresh prefix as part of gate 1.
2. **Gate 1 — the behavior contract.** Propose the **spec diff itself**: the
   actual "shall" text of the new or changed requirements, with their appended
   IDs, plus any `edge-cases.md` entries. Not prose about what you intend to
   build — the normative text, ready to land. Design choices that are observable
   *are* spec, and get settled here. **Stop for approval.** For a bug fix where
   the spec is already right and the code is wrong, there is no diff to approve:
   state the requirement the code violates and move on.
3. **Gate 1b — the tracking issue.** Once the spec is approved, search the
   repo's open issues (`gh issue list`, plus a search of closed ones) for
   anything with the **same goal**. If one exists, use it — reference its number
   from here on, do not open a second. If none exists, draft the issue title and
   body and **stop for approval**; create it with `gh issue create` only once
   confirmed. Give it a **human-friendly title** — a plain-language summary of
   the goal a maintainer can scan, not a slug, a requirement ID, or a restated
   commit subject.

   The issue must be **self-contained**: at this point the spec lives only in
   the working tree, so a reader who has only the issue cannot look a
   requirement ID up. **Always quote the full normative text** of every new
   requirement next to its ID, and list every *changed* requirement the same way
   (old → new), plus the `api-contract.md` and `edge-cases.md` entries. An ID
   with no text is useless to the reader.

   The issue body states the **goal** and the normative changes only. **Never
   put implementation detail in it** — how the code will be structured, which
   files or functions change, the chosen approach. That is part of the
   implementation, so it belongs to the plan (gate 2) and to the PR that
   describes how the issue was resolved, never to the issue.

   **Structure every issue with `##` section headers**, not a wall of prose, so
   a reader can scan it: a `## Background` (or `## Why`) stating the problem and
   context, a `## Scope` (or the requirement changes) naming what is in scope,
   and a `## Goal` stating the outcome. Add further sections as the issue
   warrants. Keep long enumerations compact (grouped ID ranges, not one
   paragraph per item). The same structured, header-per-section shape applies to
   PR bodies (gate 3).
4. **Write the spec into the working tree.** Do not mark it "unfinished" in the
   file — the file only ever contains normative text. The plan tracks what is
   not yet backed by a passing test.
5. **Gate 2 — the implementation plan.** Stages, file-level steps, a table
   mapping each new requirement ID to the test that will pin it, and a
   **Verification** section naming how the change will be exercised. **Stop for
   approval.**
6. **Implement, stage by stage.** A stage is a **green checkpoint**: it
   compiles, `cargo test` passes, `cargo clippy -- -D warnings` passes. **Commit
   every green stage** — that is what makes the plan resumable after an
   interrupted session. Stage commits are branch-local scaffolding and are
   squashed away on merge, so keep their messages cheap; the squash message is
   the one that must carry the requirement IDs and the why. The spec is the
   first stage, hence the first commit — legal on a branch, never on `main`.
   Every new or changed requirement ships with at least one test whose doc
   comment cites its ID (`/// BD-R-012 — …`). Existing tests carry IDs on the
   same terms: every test that pins observable behavior shall cite the
   requirement it verifies. A test of a pure internal or helper detail that no
   requirement governs may stay untagged. Where a test verifies real behavior
   that no requirement yet states, add the requirement (a normative change —
   gate 1) rather than attach a loose ID.
   The citing doc comment goes on the line **directly below** the
   `#[test]`/`#[tokio::test]` attribute, immediately above the `fn`. A given
   requirement ID appears **at most once** per test — one test verifying several
   requirements lists each once; never repeat the same ID.
   The task is not done until the plan's Verification method has actually been
   run and its outcome reported. Waiving it requires asking.
7. **Reconcile the spec.** If implementation forced the behavior to differ from
   what gate 1 approved, the "shall" text changes — that is a **normative**
   change and it **re-opens gate 1**: show the diff, say what forced it, get
   approval before committing. Fixing a wrong cross-reference or clumsy wording
   is **editorial** and needs no approval. **Always report the final spec diff**
   when you finish, so the difference between the two is visible without
   diffing by hand.
8. **Gate 3 — the pull request.** With the work done, the Verification method
   run and its outcome reported: **stop and ask whether to open a PR.** The user
   may want a manual test run of their own first — that is the point of this
   gate, so do not pre-empt it. Once they confirm, draft the PR title and body
   and **stop for approval** of that text. Give the PR a **human-friendly
   title** in the same plain-language style as the issue. The PR body is where
   the implementation lives: the why, the requirement IDs, **how the issue was
   resolved** (the approach and structure the issue deliberately omitted), the
   verification actually performed, and `Closes #<issue>` from gate 1b. Only
   then push the branch and `gh pr create`.

Merge to `main` by **squash merge**, so the branch's stage commits — including
the spec commit that briefly ran ahead of its code — never reach `main`.

## Where to look for task X

| Task touches | Read |
|---|---|
| Boards, columns, cards, card operations | [`docs/specs/board/`](./docs/specs/board/) |
| Save-file format, save/load, migration | [`docs/specs/storage/`](./docs/specs/storage/) |
| Screen layout, keybindings, commands, navigation | [`docs/specs/tui/`](./docs/specs/tui/) |
| Platforms, performance, versioning | [`docs/specs/non-functional-requirements.md`](./docs/specs/non-functional-requirements.md) |
| Crate structure, data flow | [`ARCHITECTURE.md`](./ARCHITECTURE.md) |
| Contribution workflow, conventions | [`CONTRIBUTING.md`](./CONTRIBUTING.md) |

## Build / test / lint

```sh
cargo check
cargo test
cargo clippy -- -D warnings
cargo fmt --check
```

Run these before considering work done.

## Conventions

*(TBD — record test naming, module layout, and other repo conventions here as
they're established. See ferrowl's `AGENTS.md` for the shape this section takes
once conventions exist: unit test placement/naming, port-binding rules for
network tests, formatting rules, etc.)*

## Scope boundaries — check with the user before

*(TBD — record any deliberately-fixed surfaces here as they're identified, e.g.
"expanding the save-file format" or "adding a new persistence backend.")*
