---
id: T-3610
title: "Decide how the eval scores an oracle cell whose Excel value varies between fills of the same bytes"
milestone: M9
lane: harness
status: done
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
A decision record (`make new-decision`) that says how the eval scores an oracle cell whose Excel value differs between
two genuine fills of the same input bytes. Options include keeping it a named known failure, excluding it from scoring,
or requiring N agreeing fills before a value is a target. The task applies the decision to the cells known today.

## Context
- T-3590 (financial.md "AMORDEGRC below period 1"): gen-m9 d912671baa1364aa Calc!A76 is 1 in its 2026-10-28
  excel-graph fill. Probes/amordegrc-a76-bisect-a, the same input sha256 (685de544…), is 0 on 2026-11-01, as is -h.
  The other 412 cells agree. The cell stays an F line in eval/baseline/gen-m9.txt against the 10-28 value. Ours is 0.
- Decision 0112 handled one such class, LN FYL2X near-ties that vary between fills (probes-numeric/t2143-ln-near-ties-fill1..3).
  It keeps them as named known failures and changes no scoring. Its principle is that bits which vary between fills
  are not targets. It is scoped to LN.
- probes/amordegrc-fraction-state-ai Sheet1!A1 (T-3393, decision 0284) is a second such cell. It was 0 on
  2026-10-30, and the same input sha256 (3f8ec3b8…) gives 485 in both 2026-11-05 fills: the copy -av, and an -ai
  refill that is not committed. It is an F line in eval/baseline/probes.txt against the 10-30 value. Ours is 485.
- probes/t4618-order-after-active-backward Q!A1 (T-4642) is a third such cell. It was `#NAME?` on 2026-11-06, and the
  same input sha256 (f800e58a…) reads 1 in the copies probes/t4642-backward-refill and -refill-b, filled 2026-11-07.
  Those two are one fill, from one upload. Every one-variable variant reads 1, and no rule explains the old value. It is an F
  line in eval/baseline/probes.txt against the 11-06 value. Ours is 1.
- Raised by the excel-fidelity review of T-3590 (PR #2685), MINOR finding 1.

## Acceptance criteria
- [x] A decision record states the general rule, how a cell is shown to vary (how many fills, by whom, and what
      counts as the same bytes), and where such cells are listed. Re-filling until a run matches us is ruled out.
- [x] gen-m9 d912671baa1364aa Calc!A76, probes/amordegrc-fraction-state-ai Sheet1!A1, probes/t4618-order-after-active-backward Q!A1 and decision 0112's LN cells are treated per the decision, and
      `make eval-baseline` records no regression.
- [x] `make check` green; reviewers approve.

## Out of scope
- Finding why A76 was 1 on 10-28 (T-3590 ruled out what it could).
- Changing decision 0112's LN model.

## Outcome
Decision 0358 (owner ruling, 2026-11-08): a cell a later Excel fill of the same inputs superseded, or that later fills do not reproduce, is scored against that later fill. Spec suites name a `superseded` file (eval/suites/probes, gen-m7, gen-m9 `superseded.tsv`, 55 lines); `runner/prepare.rs` `apply_superseded` scores each line's cell against the same cell of the later golden sidecar, refusing a line whose later case is not a copy (`overrides::same_inputs`) or that cites another cell. The earlier sidecars stay committed. The three variance cells this task listed pass: gen-m9 d912671baa1364aa Calc!A76 (against amordegrc-a76-bisect-a), probes/amordegrc-fraction-state-ai Sheet1!A1 (against -av) and probes/t4618-order-after-active-backward Q!A1 (against t4642-backward-refill). A cell that varies with no reproducing later fill stays a named known failure (decision 0358 item 5). Decision 0112's LN near-ties, whose three fills disagree, stay named known failures (0358 item 5). The general rule, how a cell is shown to be superseded and why re-filling until a run matches is ruled out, is 0358 item 3a.
