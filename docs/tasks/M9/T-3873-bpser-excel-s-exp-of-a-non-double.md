---
id: T-3873
title: "bpser: Excel's EXP of a non-double z = fl(a0 LN x) - L (the one residual of T-3769)"
milestone: M9
lane: research
status: todo
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
- Method: docs/research/excel-kernel-playbook.md; a new docs/research/t2464/r7-* directory importing r6's `common.py`.
  Positive controls first, a real hold-out, record negatives (decision 0056).

## Acceptance criteria
- [ ] A RESULTS.md gives a model of E for non-double z with exact / within-4-ulp / within-15-digit counts on a held-out
      set, or the negatives tried with their counts.
- [ ] statistics.md's "Incomplete beta bpser" entry is updated with what is found.
- [ ] Engine follow-ups are filed for any solved piece.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Engine changes; b0 >= 8 algdiv (T-3569); GAMMALN accuracy (T-3042).
