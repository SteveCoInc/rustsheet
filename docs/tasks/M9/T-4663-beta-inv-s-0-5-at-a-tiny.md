---
id: T-4663
title: "BETA.INV's 0.5 at a tiny shape and a high p: find what stops Excel's search at its start (T-2808 split)"
milestone: M9
lane: functions-stat
status: todo
priority: P3
size: M
tier: opus
depends_on: [T-4676]
defer: phase-1b
---

## Goal
BETA.INV and BETAINV give Excel's 0.5 where its search stops at its start (a shape of 1E-300 or DBL_MIN),
from a structural rule that also says where Excel is `#NUM!` instead.

## Context
Split out of T-2808 (2026-11-06), which implemented the underflow `#NUM!` and found no rule for the 0.5.
Evidence: docs/excel-semantics/statistics.md "BETAINV is #NUM! where its quantile underflows, but 0.5 at a tiny
alpha and a high p (T-2435)" and "BETA.INV's Gate A table: ... 0.5 at a DBL_MIN shape" (class 3). Owner rule:
"do not overfit any function values": implement only a stated structural rule, with a probe that
discriminates it and predictions committed before the fill (excel-probe skill).
Candidate (T-2808, refuted as stated): in AS 109's own orientation (p, or 1 - p with the shapes swapped above
1/2), Excel is 0.5 where AS 109's first approximation is 1 (its branch `1 - exp((ln((1 - a) q) + ln B)/q)`,
the exp underflowing) and the search is reset to the midpoint, as R's old AS 109 port did for a start outside
(fpu, 1 - eps). It fits 12 of the 13 such cells on file, but BETAINV/s1 A125
(`BETAINV(0.826804627471484,2.22507385850721E-308,0.478796631378503)`) is `#NUM!` with the same start, while
A135 (0.692122660100602, DBL_MIN, 0.517627748014771) is 0.5. AS 109's first Newton step at 0.5 is finite at
both (ln of the density factor 707.34 and 707.37), so "the first step cannot move" does not separate them
either. A probe of BETA.DIST(0.5, the other shape, DBL_MIN) at A125's and A135's shapes, and of BETA.INV on a
grid of p between them, would show whether Excel's cdf at the start decides it.
Code: crates/rustsheet-engine/src/functions/statistical/beta.rs `beta_quantile`, inverse_beta.rs.

Cells (known failures):
- probes/t2435-dist-extremes S!A73, A111
- eval/accuracy/known-failures.tsv: BETA.INV/s1 A115, A118, A125, A128, A138; BETAINV/s1 A115, A118, A128,
  A135, A138

Needs an oracle sweep (gap triage 2026-11-06): the cached cells cannot settle the mechanism, so this task is not
attempted blind. It waits for T-4676, which fills `probes-numeric/t4676-beta-inv-tiny-shape-half`
with predictions committed before the fill. Read that case and its statistics.md table first.

## Acceptance criteria
- [ ] The cells above pass from a structural rule evidenced by a discriminating probe, or stay known
      failures with the reason recorded in statistics.md.
- [ ] R4 guards (reviewer-excel-fidelity check 7, decision 0188), for any rule chosen by scoring candidates against
      oracle cells: train and hold-out counts side by side (bit-exact, within 4 ulp, within 15 digits) for the chosen
      rule and the correctly rounded one, on a split of about 30% stated before scoring; a fresh sweep oracle-filled
      after the rule is frozen, with its result recorded here; the Gate A envelope count on that sweep, listing every
      point outside it; a `FITTED` tag and its fitted range on any fitted constant (none for a structural model);
      the reproducibility floor met; no branch that keeps matched cells passing without the rule (no keep-matched
      branch).
- [ ] `make check` green; reviewers approve.

## Out of scope
- F.INV.RT's far-tail saturation (T-4662).
- p above 1/2 at a tiny first shape, Excel 2^-53 (T-3351).
