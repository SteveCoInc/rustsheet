---
id: T-3873
title: "bpser: Excel's EXP of a non-double z = fl(a0 LN x) - L (the one residual of T-3769)"
milestone: M9
lane: research
status: done
priority: P2
size: M
tier: opus
depends_on: []
campaign: accuracy
---

## Goal
Excel's E = EXP(z) inside bpser is modelled for z = fl(a0 LN x) - L that is not a double, or the forms tried are
recorded with their counts.

## Context
- Follow-up of T-3769 (docs/research/t2464/r6-bpser-xlinked/RESULTS.md). At a0 < 1 < b0 < 8, GL53's bpser is
  bit-exact wherever z = alx - L is a double: 0 misses of 1,143 cells on t3570-r5-bpser-lngamma-beta, 2 of 1,923 on
  t3769-r6-bpser-beta. Every other miss (417 of 777 and 1,317 of 2,685) has a non-double z, nearly always halfway
  between two doubles (`nonrepr.out`). That one effect is both of T-3570's residuals, the x-linked one and the a0-linked
  one.
- Ruled out in T-3769: z rounded nearest, down or up (no rule fits every shape; `zround.out`, `score_z.out`); L kept
  extended and other groupings; product splits; series forms and stop rules. Excel's I is matched on some tie cells by
  EXP of the extended z, on others by EXP of z rounded down, and on some by neither.
- Ideas: the EXP routine's reduction t = z·log2(e) at 64 bits with a different constant or rounding for arguments with
  bits beyond 53; F2XM1's own error on f, which is invisible for double z; an E = EXP(zh)·(1 + zl) split. The worksheet
  cannot pass a non-double argument to EXP, so any model is scored through BETA.DIST cells. The 768 non-double-z cells
  at r6's a0 = 0.5 shapes and (4.75, 0.7) are a ready tune set. r5's non-double cells are a separate hold-out.
- Also from T-3570's review, moved here by T-3769: re-score r2-bpser-small's old w2f (0.5, 6.25) shapes with measured
  GAMMALN, and compare GL53 with the gamln1 form at small a0, b0 <= 1 and b0 >= 8. Split cells by double z first: only
  the double-z cells can judge L.
- When editing statistics.md's bpser entry, also apply the wording fixes deferred on #2897 and #2906 (their PR
  comments). Most important: g is Excel's GAMMALN(a0); the GAMMALN(fl(1 + a0)) - LN a0 construction equals it only
  below 0.7.
- Method: docs/research/excel-kernel-playbook.md; a new docs/research/t2464/r7-* directory importing r6's `common.py`.
  Positive controls first, a real hold-out, record negatives (decision 0056).

## Acceptance criteria
- [x] A RESULTS.md gives a model of E for non-double z with exact / within-4-ulp / within-15-digit counts on a held-out
      set, or the negatives tried with their counts.
- [x] statistics.md's "Incomplete beta bpser" entry is updated with what is found.
- [x] Engine follow-ups are filed for any solved piece.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Engine changes; b0 >= 8 algdiv (T-3569); GAMMALN accuracy (T-3042).

## Outcome
Solved (docs/research/t2464/r7-bpser-nondouble-z/RESULTS.md). Excel splits a non-double z = fl(a0 LN x) - L into
zh = fl(z) (nearest, ties to even) and zl = z - zh, and computes I = fl(fl(EXP(zh)·s)·fl(1 + zl)), every operation a
stored double. fl(1 + zl) equals Excel's EXP(zl) on every such zl. Where z is a double this is GL53's form unchanged.
- Chosen on r6's tune shapes (1,546 of 1,546 non-double-z cells, against 808 for EXP of the extended z), then frozen.
  Hold-out and fresh sets, scored after the freeze: r6 b0 = 4.75 1,139 of 1,139; r5 777 of 777 (all 1,920 cells
  exact); T-3771's t3570-r5-bpser-lg 2,410 of 2,413; old w2f (0.5, 6.25) with measured GAMMALN 1,079 of 1,079;
  r2-bpser-small b0 <= 1 3,088 of 3,089 (shapes and runs). Split model exact / 4 ulp / 15 digits on r6's hold-out:
  1,536 / 1,536 / 1,536 of 1,536.
- R4 record (RESULTS.md section 4): R4.1 hold-out against CR, exact / 4 ulp / 15 digits: tune CR 534 / 2,467 / 2,822,
  split 3,070 / 3,072 / 3,072; hold-out CR 178 / 1,353 / 1,536, split 1,536 / 1,536 / 1,536. No margin was stated
  before scoring. The R4.2 fresh probe (filled after the freeze) and its Gate A envelope are T-3879's acceptance items.
  These are excel-graph bits; the effect (1-2 ulp) is below the web-against-desktop floor.
- Negatives with counts: 14 ways of carrying zl times 4 roundings of zh (`forms_*.out`). Every other order or
  precision loses 226-743 of the 1,546 tune cells.
- Also done: r2's old w2f shapes re-scored with measured GAMMALN (the "prefer g1ln53" reading came from the unsplit
  z); GL53 against the gamln1 g at b0 <= 1 (GL53 2,211 of 2,212 against 1,759 on the shapes probe). b0 >= 8 is still
  blocked on its L (T-3569).
- statistics.md's bpser entry: the rule now has the split; the deferred wording fixes of #2897 (158 of 172 model-high;
  the 2 double-z misses are s-side) and #2906 (g is GAMMALN(a0); the construction equals it below 0.7; "not LN, s or
  L" by elimination) are applied.
- Engine follow-up: T-3879 (functions-stat). No eval delta (docs only).
