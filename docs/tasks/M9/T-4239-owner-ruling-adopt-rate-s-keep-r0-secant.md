---
id: T-4239
title: "Owner ruling: adopt RATE's keep-r0 secant model (PR #3019) despite the five error-against-number points it moves out"
milestone: M9
lane: functions-financial
status: blocked
priority: P2
size: S
tier: opus
depends_on: [T-3963]
---

## Goal
A ruling on T-3963's keep-r0 model of RATE's secant (financial.md "RATE keeps its older iterate when a step leaves
the residual unchanged"): adopt it, with the points it moves out of the Gate A envelope accepted, or leave the merged
rule (T-3698's restart) as it is.

## Context
- The model is implemented on branch `agent/functions-financial/T-3963-rate-noise-floor-iterates` (draft PR #3019,
  with its regression record, decision 0260 on that branch). It is not on `main`; T-3963 landed its finding, probe
  and research tools without the engine change.
- What it gains (T-3963 Outcome; financial.md entry above):
  - bit-exact on the 2,400 literal `=RATE(` oracle cells of probes and probes-numeric: 1,914 -> 1,992; train
    1,357 -> 1,416, test 557 -> 576 on the fit lab's split;
  - on the blind fresh probe probes/t3963-rate-keep-r0-fresh (240 calls where the two models differ): bit-exact 59
    against 9; cells of the wrong type 8 against 21;
  - on probes/t3906-rate-restart-oos it brings A17, A194 and A195 back inside. A194 is the real exit decision 0265
    accepted for the merged rule.
- What it costs:
  - 37 gated cells the merged rule has bit-exact (decision 0260 on the branch: 19 within 4 ulp, 13 beyond, 5 of the
    wrong type);
  - five error-against-number points that the merged rule has inside the envelope and the model moves out, which
    decision 0265 does not cover ("an error against a number still fails, as R2 says"). Checked on the branch build,
    2026-11-04:
    - t3906-rate-restart-oos A143, `RATE(2,-133302979097.438,275948270243.772,-3822300.03364782,1,0.442552879652828)`:
      Excel -0.06546463553005233, the model `#NUM!`;
    - t3963-rate-keep-r0-fresh A59 and A60 (`RATE(40,1666.174974724,217139.066904346,-79546.9586107545,0,g)`), A73
      (`RATE(44,962.6445623078,-9.045250526913,-42372.09571075,0,-0.217304)`) and A188
      (`RATE(1,-304238646703.053,303193515237.441,2004026157.19246,1,0.0343348)`): Excel `#NUM!`, the model a number.
  - t3906 A15, A64 and A121 also move out on the number side. They miss Excel's 15 digits under either judgement of
    decision 0265 (A15 and A64 are nper-1 calls with pv = -pmt, fv = 0, whose residual is rounding noise for every
    rate), so 0265 does not bring them back either.
- Reviewer-excel-fidelity check 7 ("a fit that moves a point out of the envelope fails") blocks the model on these
  points, and decision 0188 is the owner's. So only the owner can accept them, as decision 0265 accepted A194 for the
  merged rule. On the fresh probe the model moves 4 error points out and 17 in; on t3906, 1 out (A143) and 3 in.

## Blocked because
Blocked because: owner ruling. Choose one:
1. **Adopt keep-r0** with the five error-against-number points (and t3906 A15, A64, A121) accepted. Then an opus run
   resumes PR #3019 from its "Open items" (T-3963's Handoff on that branch), runs `make catch-up PR=3019`, the full
   reviewer panel, and enqueues it.
2. **Keep the merged rule.** Close PR #3019; T-3970 is then moot and is closed too.

## Acceptance criteria
- [ ] The owner's ruling recorded (a decision record, or this task's Outcome citing it).
- [ ] Option 1: PR #3019 lands with decision 0260 and the fresh probe's baseline; option 2: #3019 and T-3970 closed.
- [ ] `make check` green; reviewers approve.

## Known failures owned
Taken over from T-3963 when its findings landed without the model:
- every cell of probes/t3698-rate-flat-restart that is off Excel;
- every cell of probes/t3698-rate-flat-restart-fresh that is off Excel;
- every cell of probes/t3906-rate-restart-oos that is off Excel;
- every cell of probes/t3850-rate-same-state that is off Excel;
- every cell of probes/t3963-rate-keep-r0-fresh that is off Excel.
- The cases are named here on lines with no cell address, so this task owns every cell of them.

## Out of scope
- Noise-only nper-1/2 restarts under keep-r0 (T-3970) and the zero-step `#NUM!` cells (T-3424).
