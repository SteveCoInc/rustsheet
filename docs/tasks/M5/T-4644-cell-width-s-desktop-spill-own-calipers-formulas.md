---
id: T-4644
title: "CELL(\"width\")'s desktop spill: own calipers formulas_information Sheet1 O140:O160 (FALSE, the second element of desktop's {width, FALSE}) under decision 0308 item 4"
milestone: M5
lane: functions-logic
status: in-progress
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
- [ ] The Outcome states whether O140:O160 count toward any Phase 1 or exit measure today (`make phase1-exit`, the
      `v15_checkpoint` rate of CELL, the calipers row), citing the report fields read.
- [ ] If they count: 21 lines in eval/milestones/phase1-exceptions.tsv under decision 0308 owned by T-4373, and T-4373's
      cell list names O140:O160, with the web evidence 0308 item 4 requires cited in the PR (or a decision amendment if
      the refused canary means no sidecar can exist). If they do not count: T-4373's cell list names them anyway, so a
      future desktop oracle refills them with the N cells, and the Outcome says no exception line was needed.
- [ ] The 21 known-failure lines stay in eval/baseline/calipers.txt (no engine change; the engine matches the web), and
      no new line appears there.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Desktop `CELL` values (decision 0266).
- Making `--all-off` see spill cells (T-4486's question).
