---
id: T-3982
title: "IMSECH's two T-3930 leftover texts (t3721 C386 complex, C390 real): find Excel's cosh where libm's misses"
milestone: M9
lane: functions-engineering
status: todo
priority: P3
size: S
tier: opus
depends_on: [T-3981]
campaign: visible
---

## Goal
The two IMSECH texts no open task owns print Excel's digits, or are recorded as accepted residue with ranked
candidates scored on T-3981's sweep. These are the last 2 visibly-off cells `exit_owners.py --all-off` lists with no
owner at all.

## Context
- T-3992 (excel-graph fill-to-fill stability, 2026-11-03): its kernel was not measured; see docs/research/t3992/README.md.
- Gap triage 2026-11-03 (`make eval` at 42b10ad4, then `python3 scripts/exit_owners.py --all-off`): 63 visibly-off cells
  have no open non-deferred owner. 61 are owned only by `defer: phase-1b` tasks (T-3937, T-3640, T-2221, T-3474,
  T-3527, ...). These 2 have no owner of any kind:
  - probes-numeric/t3721-im-hyperbolic-small-x-fresh-sweep.case.json (2): S!C386, C390
  - probes-numeric/t3981-imsech-real-band-sweep.case.json (25 IMSECH) and
    probes-numeric/t3981-imsech-real-band-sweep-hold.case.json (7 IMSECH): the libm cosh misses at
    0.004 <= |x| <= 0.01, where the `1/u` form fits 450 of 450 (engineering.md, T-3981 sweep)
  - C386 `IMSECH("-0.00529114829570232-0.0605498694819928i")`: Excel "1.0018218211334-0.00032135183420584i",
    ours "...-0.000321351834205841i". Excel's imaginary part prints 14 digits, so its double rounds to ...5840 at 15
    digits and ours to ...5841: one unit in the 15th digit. T-3930 found every cosh candidate prints ...841, so
    the miss is in the complex quotient (the `cos y`/`sin y` products, or the |cosh z|^2 denominator), not in cosh x.
  - C390 `IMSECH("-0.00802240437783925")` (real): Excel "0.99996782137691", ours "0.999967821376909".
- T-3930 (done) left both UNVERIFIED: docs/excel-semantics/engineering.md "IMSINH, IMCOSH, IMSECH and IMCSCH take
  libm's cosh below |x| 1E-4". Code: crates/rustsheet-engine/src/functions/engineering/complex/trig.rs
  (`inverse_exp_sinh_cosh`, `LIBM_COSH_BELOW`, `INVERSE_EXP_LOW`; a real IMSECH at |x| <= `INVERSE_EXP_LOW` takes
  libm's `1/cosh`, T-3609).
- Scored at triage over all 369 cached real IMSECH texts with 0 < |x| <= 0.01 (t3609-imsech-imcsch-small-real-sweep,
  t3589-im-hyperbolic-small-x-sweep, t3721 fresh sweep, t3930-r4-im-small-x-cosh): libm `1/cosh` 368, the `1/u`
  form's `1/cosh` 362, `2u/(1 + u^2)` 347. C390 is the only real text libm misses, and the form matches it. With one
  miss there is nothing to fit: **Needs an oracle sweep**, T-3981 (research, the [0.004, 0.01] band with worksheet
  COSH companions and a hold-out). Excel's cosh may be Microsoft's CRT cosh, which differs from glibc's at isolated
  points; the COSH and 1/COSH columns of T-3981 tell a kernel difference from a reciprocal one.
- Numeric fitting: tier opus, time-boxed (decision 0056). If no rule separates C390 on T-3981's points, the answer is
  the record, not a keep-matched branch.

## Visible-parity stage
`campaign: visible` (decisions 0083, 0114): Excel's text is the target and the error is a last-digit kernel difference,
not a semantic rule. Done when C386 and C390 equal Excel's text and no IM* oracle cell changes from pass to fail, or
when the Outcome records each one as UNVERIFIED residue with the candidates ranked on T-3981's train and hold-out sets
(then the cells move to a `defer: phase-1b` follow-up task, not left unowned).

## Acceptance criteria
- [ ] probes-numeric/t3721-im-hyperbolic-small-x-fresh-sweep S!C386 and S!C390 match Excel's text, or the Outcome lists
      each left over with the ranked candidates and a `defer: phase-1b` follow-up task owns it.
- [ ] Every other IMCOSH, IMCSCH, IMSECH and IMSINH oracle cell keeps passing: the fit lab's `im-hyperbolic*`,
      `im-small-x-complex*` and `im-small-imag` sets, the t3589, t3609, t3720, t3721, t3930 and T-3981 cases, and gen-m9.
      These suites are not baselined; `make eval-check` shows 0 regressions and no new eval/baseline/ line.
- [ ] R4 guards (reviewer-excel-fidelity check 7, decision 0188) for any kernel change:
  - [ ] Hold-out (R4.1): train and hold-out counts side by side (bit-exact text, within 15 digits) for each candidate,
        T-3981's hold-out case never used to choose the rule;
  - [ ] Fresh sweep (R4.2): a new excel-graph case drawn after the rule is frozen (committed generator and seed),
        with its score;
  - [ ] Gate A envelope (R2): the result on the fresh sweep (texts outside 1e-9 relative of the 50-digit value: 0
        expected; any error-against-number cell named);
  - [ ] `FITTED(T-3982, <n> cells, <|x| range>)` tag on any fitted cut or kernel switch, with `quality/fitted.json`
        updated by `scripts/check_fitted.py --update`;
  - [ ] Reproducibility floor (R4.4): each fixed cell is a whole printed digit, stated in the Outcome;
  - [ ] No keep-matched branch: no rule keyed on C386's or C390's argument, or a range narrower than the evidence
        brackets.
- [ ] engineering.md's "IMSINH, IMCOSH, IMSECH and IMCSCH take libm's cosh below |x| 1E-4" entry records the result,
      with the 368-of-369 triage count, the cells and the T-3981 scores.
- [ ] `make check` green; reviewers approve.

## Out of scope
- |x| above 0.01, the sinh series (T-3721), IMCSCH (T-3912, T-3914), and IMSECH at x = +/-354.03 (T-3914).
