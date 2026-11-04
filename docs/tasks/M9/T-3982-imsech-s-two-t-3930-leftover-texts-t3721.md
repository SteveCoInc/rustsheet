---
id: T-3982
title: "IMSECH's two T-3930 leftover texts (t3721 C386 complex, C390 real): find Excel's cosh where libm's misses"
milestone: M9
lane: functions-engineering
status: done
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
- [x] probes-numeric/t3721-im-hyperbolic-small-x-fresh-sweep S!C386 and S!C390 match Excel's text, or the Outcome lists
      each left over with the ranked candidates and a `defer: phase-1b` follow-up task owns it.
- [x] Every other IMCOSH, IMCSCH, IMSECH and IMSINH oracle cell keeps passing: the fit lab's `im-hyperbolic*`,
      `im-small-x-complex*` and `im-small-imag` sets, the t3589, t3609, t3720, t3721, t3930 and T-3981 cases, and gen-m9.
      These suites are not baselined; `make eval-check` shows 0 regressions and no new eval/baseline/ line.
- [x] R4 guards (reviewer-excel-fidelity check 7, decision 0188) for any kernel change:
  - [x] Hold-out (R4.1): train and hold-out counts side by side (bit-exact text, within 15 digits) for each candidate,
        T-3981's hold-out case never used to choose the rule;
  - [x] Fresh sweep (R4.2): a new excel-graph case drawn after the rule is frozen (committed generator and seed),
        with its score;
  - [x] Gate A envelope (R2): the result on the fresh sweep (texts outside 1e-9 relative of the 50-digit value: 0
        expected; any error-against-number cell named);
  - [x] `FITTED(T-3982, <n> cells, <|x| range>)` tag on any fitted cut or kernel switch, with `quality/fitted.json`
        updated by `scripts/check_fitted.py --update`;
  - [x] Reproducibility floor (R4.4): each fixed cell is a whole printed digit, stated in the Outcome;
  - [x] No keep-matched branch: no rule keyed on C386's or C390's argument, or a range narrower than the evidence
        brackets.
- [x] engineering.md's "IMSINH, IMCOSH, IMSECH and IMCSCH take libm's cosh below |x| 1E-4" entry records the result,
      with the 368-of-369 triage count, the cells and the T-3981 scores.
- [x] `make check` green; reviewers approve.

## Out of scope
- |x| above 0.01, the sinh series (T-3721), IMCSCH (T-3912, T-3914), and IMSECH at x = +/-354.03 (T-3914).

## Outcome
- **C390 fixed.** A real IMSECH is now 1 over IMCOSH's own cosh x (`inverse_exp_sinh_cosh`: libm's below
  `LIBM_COSH_BELOW` = 1E-4, the `1/u` form above), not libm's `1/cosh` up to 0.01 (T-3609). The change removes a branch
  in `sech_parts` (crates/rustsheet-engine/src/functions/engineering/complex/trig.rs); no new constant. C390
  `IMSECH("-0.00802240437783925")` is Excel's 0.99996782137691 (libm's ...909): a whole 15th printed digit (R4.4).
- **Candidates ranked (R4.1)** over the 825 cached real IMSECH texts at 0 < |x| <= 0.0105 (they span |x| 2E-309..0.00999:
  t3589, t3609 and its tiny-real probe, t3643, t3721, t3981 and its hold-out). Bit-exact text and within 4 ulps of
  Excel's printed value:
  | candidate | train text (675) | train <= 4 ulp | hold-out text (150) | hold-out <= 4 ulp |
  |---|---|---|---|---|
  | IMCOSH's cosh (libm < 1E-4, form above): chosen | 675 | 675 | 150 | 150 |
  | `1/u` form's cosh everywhere | 669 | 669 | 150 | 150 |
  | libm `1/cosh` (old rule) | 649 | 649 | 143 | 143 |
  | `1/((u + 1/u)/2)` with u = EXP(x) | 660 | – | 141 | – |
  | `2u/(1 + u²)` with u = EXP(x) | 615 | – | 138 | – |
  | correctly rounded sech (mpmath, 50 digits): reference | 616 | 618 | 135 | 135 |
  The hold-out is t3981-imsech-real-band-sweep-hold, never used to choose. The rule follows from the train misses
  alone: the form misses only at |x| <= 9.70E-05 and libm only at |x| >= 0.0042, so the existing T-3930 cut
  separates them. Margin: none was written down before scoring (an omission in this record). Taking T-3930's 5
  points, the chosen rule beats the correctly rounded reference by 15 on the hold-out (150 against 135) and by 59 on
  train. Within 4 ulps equals the text count for every kernel candidate because a miss here is one printed digit,
  which is more than 4 ulps at these magnitudes.
- **Fresh sweep (R4.2)**, drawn after the freeze: probes-numeric/t3982-r4-imsech-real-cosh (docs/research/t3982/gen_sweep.py,
  seed 3982, 300 real x, |x| log-uniform in 1E-6..0.0105, 116 in the 1E-4..0.004 gap; excel-graph): IMSECH 300/300,
  IMCOSH 300/300; libm `1/cosh` would give 296 (misses at |x| 1.13E-04, 1.56E-04, 3.02E-04, 0.0034), correctly
  rounded sech 287.
- **Gate A envelope (R2)** on the fresh sweep: 0 texts outside 1E-9 relative of the 50-digit sech (mpmath); no error
  cells.
- **FITTED:** `LIBM_COSH_BELOW`'s tag is now `FITTED(T-3982, 864 cells, |x| 2E-309..0.01; cut bracketed
  9.70E-05..1.09E-04)` (the T-3930 39 cells plus the 825 real texts); the cut is unchanged, the tag count stays 90, so
  quality/fitted.json needs no update. No keep-matched branch: nothing keyed on C386's or C390's argument.
- **C386 left over**, UNVERIFIED, owned by T-4012 (`defer: phase-1b`): the parse is exact and no quotient variant
  gives Excel's ...584i without losses on the 1040 complex IMSECH texts at |x| <= 0.0105 (current 1003; best
  alternatives +1 -30 and +1 -41; table in T-4012).
- **Eval:** t3721 fresh sweep 1599 of 1600 (was 1598), t3981 band sweep 1200/1200 (was 1175), hold-out 600/600 (was
  593), new t3982 case 600/600. `make eval-check`: 0 regressions in every gated suite (gen-m9 included), no
  eval/baseline/ line (probes-numeric is ungated). The only IMSECH texts of probes and probes-numeric that fail after
  the change are C386 and four at |x| 346..354 (T-3914), outside the changed domain (real x, 1E-4 <= |x| <= 0.01).
  engineering.md records the result.
