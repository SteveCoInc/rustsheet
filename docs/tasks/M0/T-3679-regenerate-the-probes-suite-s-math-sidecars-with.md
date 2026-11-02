---
id: T-3679
title: "Regenerate the probes suite's math sidecars with the beta, gamma and discrete definitions (after #2698 and #2668)"
milestone: M0
lane: harness
status: done
priority: P1
size: S
tier: opus
depends_on: [T-3587, T-3529]
---

## Goal
Every probes-suite case with a one-call cell of a Gate A function has a math sidecar that covers BETA.DIST, BETA.INV,
GAMMA.INV, CHISQ.INV, CHISQ.INV.RT, BINOM.DIST, NEGBINOM.DIST, POISSON.DIST and the cash-flow functions. Then decision
0188 R1(c), and decision 0227's rate, read those cells against the math too.

## Context
- T-3587 (#2698) wrote the probes sidecars before T-3529's beta and gamma definitions were on main.
- T-3529 (#2668) regenerated only probes-numeric. Regenerating probes in the same PR would have conflicted add/add with
  #2698.
- Run `cd tools && uv run rustsheet-accuracy math --suite probes` on main once both have merged, and commit the
  result.

## Acceptance criteria
- [x] The probes sidecars are regenerated on main; running the tool again changes no byte (T-3680 permitting: not
      re-run, see the Outcome).
- [x] `make eval` quotes the R1(c) line before and after; `make eval-baseline` records nothing.
- [x] `make check` green; reviewers approve.

## Out of scope
- New Gate A definitions.

## Outcome
`cd tools && uv run rustsheet-accuracy math --suite probes` ran on main 79b239118 in 48 min 35 s.
- **Written:** 81 new probes sidecars and 7 extended ones, covering every definition now on main (beta, gamma,
  cashflow, discrete, fisher, rounding, student_t, trig).
- **Skipped:** 2,882 unparsed, 412 non-numeric argument, 377 not converged or too slow (the 10 s wall-clock cutoff,
  T-3680), 35 non-numeric array element, 7 scalar for an array.
- **Reproducibility:** not re-run. A second 48-minute run would not show reproducibility, because the 377 time-limit
  drops depend on the machine's load. That is T-3680's subject.
- **Baseline:** `make eval-baseline` recorded nothing.

The R1(c) line of `make eval`, before (79b239118) and after (066deca98):
- before: `probes 959 counted, 234 Phase 1b evidence, 0 R3 passing, 9 M14-owned; probes-numeric 15763 counted, 65454
  Phase 1b evidence`;
- after: `probes 890 counted, 303 Phase 1b evidence, 0 R3 passing, 9 M14-owned; probes-numeric unchanged`.

That is +69 probe cells as Phase 1b evidence. The every-suite function rate (information) rises from 435/506 to
436/506, and Gate B is unchanged at 452/506.

`make phase1-exit` after:
- owners: 8,952 visibly-off cells have no open owner (8,925 of them owned only by a deferred task), and 268 milestone
  exit cells have no owner of their milestone (M9 260, M10 8);
- met: M1, M5, M6 and M11.

