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
- [x] The probes sidecars are regenerated on main; running the tool again changes no byte (T-3680 permitting).
- [x] `make eval` quotes the R1(c) line before and after; `make eval-baseline` records nothing.
- [ ] `make check` green; reviewers approve.

## Out of scope
- New Gate A definitions.

## Outcome
`cd tools && uv run rustsheet-accuracy math --suite probes` on main 73aefa3a ran for 45 minutes on 4 workers. It shows the probes
sidecars already carry the beta, gamma and discrete definitions: `rustsheet-accuracy math --check --suite probes` reports
`current` before and after the run, and no regenerated cell's value differs from main's. The run changed four sidecars:
- **Committed (deterministic):**
  - irr-guess-edges P!A30 `=IRR({0,0,0})` moves from a math error ("flows of one sign") to `skipped` ("no single
    value"), the current cash-flow definition's answer.
  - sheet-qualified-ref-error-text-rename: only `input_sha256` changes; its case file changed after the sidecar was written.
- **Not committed (T-3680 wall-clock churn):** t3350-chisq-inv-huge-df (10 points gained, 9 dropped) and
  t3505-gamma-inv-budget-band (3 gained, 25 dropped). No value changed. Points crossed `TIME_LIMIT` = 10 s in both
  directions under load. That is the machine dependence T-3680 owns, so main's fuller versions stay, as T-3529 did for
  probes-numeric. Re-running the tool therefore changes no byte only up to that churn ("T-3680 permitting").
- R1(c) line of `make eval`, the same before and after: `probes 2010 counted, 301 Phase 1b evidence, 0 R3 passing, 20 M14-owned;
  probes-numeric 4311 counted, 81154 Phase 1b evidence`. `make eval-baseline` records nothing. Eval delta: 0.
