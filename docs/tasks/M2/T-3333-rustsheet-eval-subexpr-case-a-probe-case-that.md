---
id: T-3333
title: "rustsheet-eval subexpr-case: a probe case that asks Excel for every subexpression of a visibly-off spec-case cell, and show the first one that diverges"
milestone: M2
lane: harness
status: done
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
An agent with a visibly-off cell whose formula nests several calls can find the call that goes
wrong with one oracle fill. It runs `rustsheet-eval subexpr-case <case> <Sheet!A1>`, which writes
a probe case holding each subexpression of that formula in its own cell over the same input
cells. It fills that case with `rustsheet-oracle fill`, then runs `rustsheet-eval subexpr-case
--culprit <probe case>`. That prints the innermost subexpression whose value is visibly off while
all of its children match Excel.

## Context
- The report holds 935 visibly-off Excel cells that decision 0118 attributes to no function
  (report.json `unattributed_off`, full run at 9453cfb5 on 2026-10-31). 790 are in gen-m9 and 117
  in probes. Each one's formula nests several calls, for example
  `=CHISQ.DIST(T.INV.2T(0.975,7),6,Data!E12)` or `=HYPGEOM.DIST(IRR(...),3,2,19,NOT(...))`.
  Today an owner finds the culprit by hand. It writes a probe of the inner calls, as T-2940 does
  for its 38 cells, or it guesses from the engine's own subexpression values. The guess fails
  when two calls are each a little off.
- The design is docs/architecture/synthetic-generator.md §11 stage 1 ("oracle-free
  localization") with §9.3's subexpression probing. One oracle call evaluates every node.
  Nothing implements it yet, and no task covers it (no `reducer`, `localize` or `subexpression`
  task is open).
- Reuse the engine parser through a `#[doc(hidden)]` testing hook next to `call_arguments`
  (crates/rustsheet-engine/src/call_arguments.rs, T-0077). The harness must not get a second
  formula parser (CLAUDE.md directive 3). Print nodes with the engine's own printer
  (expressions/parser/stringify.rs).
- Context traps the case must handle, or skip with a note in the case description:
  - Relative references must still point at the original cells once the text sits in another
    cell. Write the subexpression at the original cell's position on a copy of its sheet, or
    print its references absolute.
  - A node that yields a multi-cell range or array spills on its own. Under the original call it
    may have been implicitly intersected or aggregated. Check only scalar-context nodes, or wrap
    nothing and record that the node spills.
  - Nodes that read LET or LAMBDA parameters cannot leave their scope. Volatile nodes (RAND,
    NOW) cannot be compared. Skip both.
- Milestone: decision 0051 assigns an untagged probe to M2. A localization case of a gen-m9
  cell must not land in the M0-M2 exit row, so the case carries its source case's milestone tag.

## Acceptance criteria
- [x] `rustsheet-eval subexpr-case <suite/case> <Sheet!A1> [--out PATH]` writes a valid
      `rustsheet-case/v1` spec case under `eval/suites/probes/cases/` named
      `subexpr-<source stem>-<cell>`. It holds the source case's input cells, one cell per distinct
      non-literal subexpression in post-order (children first), the full formula as its last
      cell, a `check` list of those cells and the source case's milestone tag. Its description
      names the source case and cell and maps each cell to its subexpression text.
- [x] It refuses an xlsx-backed case (calipers, poi, ironcalc-calc) with a clear error and no
      panic. It also refuses a cell that holds no formula.
- [x] `--culprit <probe case>` reads the case's oracle sidecar and the engine's values. It prints
      the innermost visibly-off node whose children all match (decision 0083's rule, `closeness.rs`;
      no second rule), or `no divergent node`.
- [x] Rust tests cover: relative and absolute same-sheet references keep their targets; LET/LAMBDA
      and volatile nodes are skipped; a hand-written oracle sidecar where only an inner call is off
      gives that call as the culprit; a sidecar where everything matches gives `no divergent node`.
- [x] Demonstrated on one gen-m9 cell from report.json `unattributed_off`. The case is filled with
      excel-graph and committed, and `--culprit` output is pasted in the task Outcome. If the
      culprit is not already owned, add it to the owning task or file a follow-up.
- [x] docs/architecture/eval-harness.md "CLI" documents the command. `make check` green; reviewers approve.

## Out of scope
- Workbook ddmin, AST hoisting and literal shrinking (§11 stages 2-5). File them as follow-ups if
  this stage proves useful.
- Running it over all 935 unattributed cells, or re-owning them. That is the owners' and the
  gap-triager's work.
- xlsx-backed cases (no spec to copy the inputs from).

## Outcome
- Engine: `rustsheet_engine::subexpressions` (`#[doc(hidden)]` testing hook beside `call_arguments`,
  crates/rustsheet-engine/src/subexpressions.rs): parses with the engine parser, makes every reference absolute and
  sheet-qualified, and prints each node with the engine printer, children before parents; each node is `Literal`,
  `Reference`, `Scoped` (reads a LET/LAMBDA name), `Positional` (`ROW()`/`COLUMN()`/`SHEET()` without an argument,
  `INDIRECT`, `CELL`) or `Computed`. Parentheses, whitespace and an automatic `@` are not nodes of their own.
- Harness: `rustsheet-eval subexpr-case <suite/case> <Sheet!A1> [--out PATH]` (crates/rustsheet-eval/src/subexpr_case.rs)
  writes `probes/subexpr-<stem>-<sheet>-<cell>`: the source case's sheets plus a `Subexpr` sheet, one cell per distinct
  checkable node, whole formula last, the source's milestone tag; volatile nodes skipped through `volatile::is_volatile`.
  It refuses xlsx suites, non-formula cells and `sheet_ops` cases. `--culprit` runs the probe case and prints each
  visibly-off node (`Visible::is_off`, decision 0083) whose checked children all match, or `no divergent node`.
  Documented in docs/architecture/eval-harness.md "CLI".
- Demonstration on gen-m9/05d44f6dd6cfc567 Calc!A178 (report.json `unattributed_off`), filled with excel-graph:
  ```
  $ rustsheet-eval subexpr-case --culprit probes/subexpr-05d44f6dd6cfc567-calc-a178.case.json
  Subexpr!A6: =CHISQ.DIST(CHISQ.DIST.RT(1.5,7)%,8,(PI()=(Data!$B$9+1)))
      expected: 9.825189899531443e-9
      actual  : 9.825189899531435e-9
  ```
  Every inner node is bit-exact (A1:A5), so the outer CHISQ.DIST density (8 degrees, x 0.009823096598068504) is the
  culprit. T-3219 already owns the cell (report.json's `unattributed_off` is about function attribution, not
  ownership): the localization is added to T-3219's cell list; no new task.
- Eval delta: +1 probe case (6 cells, 5 pass); 1 new known failure (Subexpr!A6, owned by T-3219); no regressions.
- Follow-ups: none. Stages 2-5 of §11 (ddmin, hoisting, literal shrinking) not filed: wait to see whether stage 1
  is used.

## Landing notes
- PR #3856, head f772541c7 (this note's parent). Reviewers approved 933abca89 (security, design, excel-fidelity; design's
  round-1 MAJOR, the duplicate milestone-tag parser, fixed there). CI on 933abca89 failed only `eval-math-check`: the
  probe case copied all ~400 of the source case's formula cells, which need math sidecars
  (run 42342406404, job 128312977924).
- f772541c7 fixes it: `drop_unread_formulas` (subexpr_case.rs) keeps the source's constant cells and only the formula
  cells the formula reads; the case was regenerated, refilled on excel-graph (Subexpr!A1:A6 values unchanged) and given
  its math sidecar (`rustsheet-accuracy math --suite probes --function CHISQ.DIST.RT --function CHISQ.DIST`). Local
  `make eval-baseline` (no baseline change) and `make check-fast` pass.
- Next: wait for CI on the new head, re-run the three reviewers on the delta 933abca89..HEAD (behavior change in what
  the probe case copies, plus the new math sidecar and the eval-harness.md wording), then enqueue.
