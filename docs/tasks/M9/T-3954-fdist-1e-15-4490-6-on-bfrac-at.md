---
id: T-3954
title: "FDIST(1E+15,4490,6) on bfrac at x0 = 1.3E-18: Excel 1.2E-14 off the math, ours 4.6E-15 (gen-m9 8f2ac25945af87da A92, parked in deferred T-2221)"
milestone: M9
lane: functions-stat
status: todo
priority: P2
size: S
tier: opus
depends_on: [T-3953]
campaign: visible
---

## Goal
FDIST and F.DIST.RT agree with Excel to 15 significant digits where bratio dispatches to bfrac at a tiny kernel
argument, starting with gen-m9 8f2ac25945af87da A92, by reproducing Excel's arithmetic on that branch as T-3953's
sweep shows it.

## Context
- Gap triage 2026-11-03 (`make eval` on origin/main c2062e5c8, excel-graph). `scripts/exit_owners.py --all-off` lists
  this cell as visibly off and owned only by a `defer: phase-1b` task: T-2221's "Cells added from T-3198 (2026-11-03)",
  which says none of those cells is visibly off. gen-m9 is a Gate B suite (decision 0188 R1(b)); a deferred owner
  leaves the `owners` line of `make phase1-exit` unmet. This task takes the cell out of T-2221's list (T-2221 names
  this task).
- gen-m9/8f2ac25945af87da Calc!A92 `=FDIST(Data!$D7,DAYS360(31271,35828,AND(TRUE,TRUE)),6)`, Data!D7 = 1E+15 and
  DAYS360 (European) = 4490, so `FDIST(1E+15,4490,6)`: Excel 4.5060151487344243E-45, ours 4.506015148734392E-45
  (`make eval-show CASE=gen-m9/8f2ac25945af87da.case.json`, NumericClose).
- 50-digit reference (mpmath 1.3, dps 50, `betainc(3, 2245, 0, v, regularized=True)` at v = 6/(4490·1E15 + 6)):
  4.5060151487343711258E-45. Excel is +1.18E-14 relative off it, ours +4.6E-15; half a unit of Excel's 15th digit is
  1.1E-15 relative. Excel is the inaccurate side, so the stage is `visible` (reproduce Excel's arithmetic, time-boxed,
  decision 0056), not `accuracy`.
- Branch: the right tail is I_v(d2/2, d1/2) (`fisher.rs` `f_beta_arguments`); a0 = 3, b0 = 2245, x0 = 1.336E-18, λ > 0
  (no swap), b0 >= 40 and a0 <= 100, so `bratio::dispatch` gives `Branch::Bfrac`
  (`crates/rustsheet-engine/src/functions/statistical/bratio/mod.rs`, `bfrac.rs`). Candidates to score on T-3953's
  train rows: bfrac's prefactor brcomp(a0, b0, x0, y0) through Excel's ln B (T-2924's k-ulp stored ln B), the
  y0 = fl(1 - x0) = 1 rounding inside brcomp's LN terms, and the argument form v against 1/(1 + d1·X/d2) (T-2847's w5
  form, which moved F.TEST's fl(a·LN y) at a similar tiny y).
- Needs an oracle sweep: one gen-m9 cell cannot pick a form (decision 0188 R4). T-3953 fills
  probes-numeric `t3953-f-tail-bfrac-tiny` with a hold-out; this task waits for it.

Cells this task owns:
- gen-m9/8f2ac25945af87da.case.json (1): Calc!A92

Once T-3953 adds probes-numeric `t3953-f-tail-bfrac-tiny`, every visibly-off cell of that case is this task's too.
Fit on that case's column A (F.DIST.RT) only, on the fit lab's per-cell hold-out (369 A cells, marked `/hold/` in the
hypotheses), with the default margin of 5 percentage points, fixed by T-3953 before any candidate is scored. Columns B,
C and D repeat A's value on most rows and are diagnostics; D's v is a 15-digit literal, not the exact x0
(statistics.md "F right tail on bfrac at a tiny kernel argument").

## Visible-parity stage
`campaign: visible` (decisions 0083, 0114). Done when gen-m9 8f2ac25945af87da A92 and the visibly-off cells of
t3953-f-tail-bfrac-tiny are within half a unit of the 15th significant digit of Excel's value, with no other F, BETA.DIST,
BINOM.DIST or NEGBINOM.DIST cell going from within 15 digits to visibly off. Bit-level criteria still unmet then move
to a follow-up with `defer: phase-1b`.

## Acceptance criteria
- [ ] gen-m9/8f2ac25945af87da Calc!A92 is within 15 digits of Excel, and the t3953 case's visibly-off count falls to 0
      or the Outcome names each cell left with its (a0, b0, x0).
- [ ] The R4 guards of reviewer-excel-fidelity check 7 (decision 0188 R4): train and hold-out counts (bit-exact, 4 ulp,
      15 digits) for the chosen form and the correctly rounded one on T-3953's split, the chosen form beating the
      correctly rounded one on hold-out; a fresh log-uniform sweep filled after the form is frozen; its Gate A envelope
      result with every point outside listed; a `FITTED(T-3954, n_cells, range)` tag and range on any fitted piece,
      applied only inside the range (none for a structural model of kind (c)); nothing fitted below the
      reproducibility floor (4 ulp web against desktop, decision 0112); no keep-matched branch.
- [ ] No new line in any eval/baseline/<suite>.txt (`make eval-baseline` refuses regressions).
- [ ] `make check` green; reviewers approve.

## Out of scope
- bfrac's Lentz region near the mean (T-3913), basym (T-3597), the L140 branches (T-3198, T-3472), and the bit-level F
  residue (T-2221, Phase 1b).
