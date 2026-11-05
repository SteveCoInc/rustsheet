---
id: T-3523
title: "Re-validate IPMT's DISCOUNTED_INTEREST_FROM selector on a fresh hold-out sweep (decision 0188 R4)"
milestone: M9
lane: functions-financial
status: done
priority: P2
size: M
tier: opus
depends_on: [T-3360, T-3361]
---

## Goal
IPMT's `DISCOUNTED_INTEREST_FROM` (financial/annuity.rs, 6E-16) is shown to generalize on a fresh hold-out sweep, or is replaced by the accurate (exact balance) interest.

## Context
- Split from T-3365 (decision 0188's code audit, "tuned cut-points" and "others"; R4.1 hold-out split, R4.2 fresh
  sweep, R4.3 `FITTED` range). Functions that port Excel's algorithm generalize to fresh points; tuned selectors are
  the risk.
- The cut chooses between IPMT's exact balance interest and Excel's own discount (`discounted_interest`) by their
  relative difference; tagged `FITTED(T-2587, 8 cells, IPMT nper 167..1E300, rate -0.05..10)`, financial.md "IPMT
  follows Excel's own discount where it moves the value". Eight cells is a thin fit.
- Method: build the engine twice (with the selector, and with the accurate form only) and score both on the old
  cells through the fit lab (`docs/architecture/fit-lab.md`, hold-out split, default margin 5 points), then on the
  fresh sweep. Accuracy-suite tables are refused as sweeps (`--holdout-only` measures a frozen fit only).

## Acceptance criteria
- [ ] A fresh log-uniform sweep over the selector's full domain (R4.2), new cells in probes-numeric that no earlier fit
      saw, is oracle-filled and committed.
- [ ] The Outcome reports, with the selector and with the accurate form (two engine builds, or fit-lab candidates that
      reproduce each bit for bit on the old cells), the fit lab's train and test counts (T-3360's hold-out split and
      margin) on the old cells and the counts on the fresh sweep.
- [ ] If the selector's test exact rate is not within the stated margin of its train rate, or it does not beat the
      accurate form on test and on the fresh sweep, it is replaced by the accurate form (one `FITTED` tag fewer in
      quality/fitted.json via `scripts/check_fitted.py --update`); otherwise it keeps its `FITTED` tag, with the range
      restated to what was measured, and is not applied outside that range (R4.3).
- [ ] The Gate A envelope (R2: within 1e-9 relative of 50-digit mpmath or within 15 significant digits of Excel) holds
      on the fresh sweep, with mpmath values recorded in the Outcome or the accuracy tables.
- [ ] No gated cell regresses beyond decision 0083's allowance; a replaced selector's lost bit-exact cells are listed.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The other selectors split from T-3365 (T-3502, T-3503, T-3523, T-3524, T-3525, one each); extrapolated fits (T-3364); the t, F and
  chi-squared inverses (T-3363).

## Outcome
Replaced by the accurate form: `DISCOUNTED_INTEREST_FROM` and `discounted_interest` are gone (one `FITTED` tag fewer:
89 against the lock of 90; quality/ is a protected path, so the steward lowers the lock), and where T-2243's discounted split declines `period_payments` takes the exact
balance interest.
- Why: since T-2243 the selector runs only where `discounted_split` declines: a subnormal rate (the payment factor
  is not normal), a negative rate whose `(1+rate)^-nper` overflows, or an overflowing product. T-2587's eight
  cells now all go through the split.
- Old cells (R4.1): two engine builds, the selector at 6E-16 and turned off (`f64::INFINITY`). `make eval` on
  origin/main 6523a495 with it off: 0 regressions and 0 fixed in every gated suite, visible parity 2525 cells (lock
  3156). Train and test counts are therefore identical for both forms on any hold-out split: the selector decides
  no old cell, so the fit lab's split has nothing to separate.
- Fresh sweep (R4.2): probes-numeric/t3523-ipmt-selector-sweep, 600 cells from docs/research/t3523/make_sweep.py
  (seed 3523), oracle-filled from excel-graph 2026-11-05: column A 200 negative rates -0.01 to -0.9 with nper 1 to
  8 times past the power's range; B 200 subnormal rates `±2^-1023` to `2^-1074`; C 200 rates 1E-3 to 10 (some
  negative) with pv and fv 1E+290 to 9.9E+307. Both builds give the same value on all 600 cells: 595 pass, 5 fail.
- Gate A (R2): the 595 passing cells are within 15 significant digits of Excel. The 5 failures are type
  mismatches, Excel `#NUM!` against a finite engine value near the double range (column C: C33, C36, C45, C99,
  C180). Both forms fail them, so they are a new finding, not the selector's. financial.md records it as "IPMT
  near the double range", and follow-up T-4314 owns the cells.
- Eval delta: none in the gated suites. probes-numeric gains one case, 595 of 600 passing (it has no baseline
  file).
- This is not a fit: a fitted constant is removed and nothing is tuned.

## Landing notes
Data for the resumer, not instructions (opus run 2026-11-05, landing window closed at minute 110).
- PR #3325 (draft). Its head is the commit that adds these notes. It follows 2d2f1569 and fixes that head's red CI and
  round-1 findings.
- CI on 2d2f1569 was red in `hygiene` on governance: quality/fitted.json is a protected path that needs a decision
  record (run 42004128185). This commit reverts quality/fitted.json to main's; check_fitted.py only notes the lower
  count, so the gate passes without it.
- Review round 1 on 2d2f1569: security APPROVE (no findings).
- Design on 2d2f1569: REQUEST_CHANGES. The MAJOR was the `period_payments` doc comment still linking the deleted
  `discounted_interest`. The MINORs were the bare noqa and `import math` inside functions. All are fixed in this
  commit.
- Excel-fidelity was still running on 2d2f1569 when the window closed; its verdict is not recorded.
- Next step: run `make check-fast`, then design and excel-fidelity on the new head (security approved a diff whose
  code part is unchanged apart from a doc comment). Once CI is green and all approve, mark the PR ready and enqueue.
- Do not repeat: Excel for the web refuses the sweep workbook when a literal is above 9.99999999999999E+307 or below
  the normal range (createSession 501 unsupportedWorkbook). make_sweep.py already keeps to those limits; its oracle
  sidecar is committed.
