---
id: T-4644
title: "CELL(\"width\")'s desktop spill: own calipers formulas_information Sheet1 O140:O160 (FALSE, the second element of desktop's {width, FALSE}) under decision 0308 item 4"
milestone: M5
lane: functions-logic
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
The 21 cells calipers/roundtrip/formulas_information Sheet1 O140:O160 have an owner and a recorded ruling. They are
the spill half of desktop Excel's two-cell `CELL("width")` result. Either they join decision 0308's desktop-only CELL
cells, with the web evidence its item 4 asks for, or the Outcome records why they need no exception line. Today they
are known failures (eval/baseline/calipers.txt) that no open task names.

## Context
- Gap triage 2026-11-06 (report of `make eval` at 37389d18). `make unowned-off ALL=1` lists 0 unowned visibly-off
  cells, but a scan of every failing cell against open-task ownership (exit_owners `owned`) finds these 21 unowned.
  They are visibly off (TypeMismatch: Excel the boolean FALSE, ours empty), yet they are in neither `attributed_off` nor
  `unattributed_off`. A spill cell has no formula of its own, and `off_excel_cells` (crates/rustsheet-eval/src/attribution.rs)
  keeps only cells whose formula calls a function. So `--all-off` cannot list them. T-4486 (every failing cell no task
  names) would.
- `rustsheet-eval show calipers/roundtrip/formulas_information/golden.xlsx`: `Sheet1!O140 TypeMismatch, formula
  "=", expected FALSE, actual <empty>`, the same in each row to O160. N140:N160 hold
  `=IFERROR(CELL("width",$C$n),"Error")`. Excel caches 15 there and ours is "Error". Those 21 N cells are decision 0308
  exceptions owned by T-4373 (eval/milestones/phase1-exceptions.tsv).
- The quirk (recorded in information.md, "CELL(...) is #VALUE! on Excel for the web", T-4644 paragraph): on desktop,
  `CELL("width")` is a two-cell row. The golden's sheet XML gives N140 `<f t="array" ref="N140:O140">`, and O140 is
  `t="b"` with the value 0. Every other info type in C140:M140 has a one-cell range. The web returns `#VALUE!` for
  "width" (decision 0019's POI fill: 9 cells of poi/47804.xlsx), which does not spill, and the engine matches the web.
- Decision 0308 item 4: new cells of the same kind may be added by a later PR citing it, each with its own web
  evidence. The oracle canary refuses `CELL("width")` (information.md), so you cannot simply fill a fresh sidecar.
  Settling which evidence 0308 item 4 accepts for a spill cell, or whether an unattributed spill cell counts toward any
  Phase 1 rate at all, is the judgment call that makes this opus work. eval/milestones/ is a protected path (decision
  0044).
- No campaign key, although the cells are visibly off. Decision 0266 puts desktop-only values out of scope, and the
  engine already gives the web's answer, so no visible-parity target can be met here. The work is ownership and a
  ruling, like T-4373's.
- Cells: calipers/roundtrip/formulas_information/golden.xlsx Sheet1!O140:O160 (21 cells).

## Acceptance criteria
- [x] The Outcome states whether O140:O160 count toward any Phase 1 or exit measure today (`make phase1-exit`, the
      `v15_checkpoint` rate of CELL, the calipers row), citing the report fields read.
- [x] If they count: 21 lines in eval/milestones/phase1-exceptions.tsv under decision 0308 owned by T-4373, and T-4373's
      cell list names O140:O160, with the web evidence 0308 item 4 requires cited in the PR (or a decision amendment if
      the refused canary means no sidecar can exist). If they do not count: T-4373's cell list names them anyway, so a
      future desktop oracle refills them with the N cells, and the Outcome says no exception line was needed.
- [x] The 21 known-failure lines stay in eval/baseline/calipers.txt (no engine change; the engine matches the web), and
      no new line appears there.
- [x] `make check` green; reviewers approve.

## Out of scope
- Desktop `CELL` values (decision 0266).
- Making `--all-off` see spill cells (T-4486's question).

## Outcome (2026-11-06)
O140:O160 count toward no Phase 1 or exit measure today. So they get no exception line; T-4373 lists them so a future
desktop oracle refills them with the N cells. This was read from `make eval` at 37389d18 (main 4679d912 plus T-4534's
tsv change, calipers and poi fetched):
- The case's cells: O140 is `type_mismatch` / `visible_other`, with only an `anchor_formula`
  (`IFERROR(CELL("width",$C$5),"Error")`) and no `formula`.
- `attributed_off` (6361) and `unattributed_off` (602) hold none of the 21. `off_excel_cells`
  (crates/rustsheet-eval/src/attribution.rs) reads `formula` only.
- `v15_checkpoint` CELL: `rate_cells` 581, `rate_visible` 0, `rate_excepted` 149, `meets_rate` true. `rate_count`
  (v15_checkpoint.rs) also reads `formula` only, so the spill cells are outside CELL's rate and Gate B.
- `milestone_exit`: no row lists calipers (M11's required suite is corpus-bench, M8's are gen-m8b and the poi M8
  subset). No row's visibly-off list names a formulas_information cell.
- `make phase1-exit`: owners met (0 visibly-off cells with no open owner); M5 met.
- eval/baseline/visible-parity.txt has no calipers line (the lock covers the in-scope generated cases).
Changes:
- T-4373 names O140:O160.
- information.md: the formula is written out (`$C$5` through `$C$25`), and the entry gives column C's
  `customWidth="1"` (15.84375, sheet default 11.07421875) as the context for the UNVERIFIED FALSE.
- The 21 lines stay in eval/baseline/calipers.txt; no engine change.
If T-4486 makes the measures see spill cells, these cells would need a decision 0308 item 4 line. The canary refuses
`CELL("width")`, so that line would need a decision amendment for its evidence.
Eval delta: none.
