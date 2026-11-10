---
id: T-5108
title: "FVSCHEDULE: score the val2 FVS probes (Excel for the web filled) against the overflow, exponent-range and product-rounding hypotheses"
milestone: M9
lane: research
status: todo
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
Each FVSCHEDULE hypothesis of the FVS probes (what the overflowing call is, the overflow cut, the retained exponent
range, zero and unit factors, underflow, plain versus twice-rounded product) is scored against the filled Excel values,
the confirmed rules are recorded in docs/excel-semantics/financial.md, and an implementation task is filed for each.

## Context
Owns: probes/val2-fvs-01, probes/val2-fvs-02 and probes/val2-fvs-03 (every visibly-off cell of these cases).
- Probes: the val2 batch of docs/research/probe-requests-2026-11-08.md (questions, hypotheses, model predictions) and
  docs/research/probe-handoff-2026-11-08.md (how it was filled, CAL-01 result). Filled on excel-graph 2026-11-09 by the
  PR that filed this task.
- CAL-01 passed on every anchor (the build reproduces the cached web corpus), but in-sheet `SUM(x,-y)*2^52` residuals
  snap to 0 (numbers.md, the SUM near-zero snap): read residuals from the raw value cells, never from a SUM column.
- No engine change lands until a probe confirms its rule; no fitted special cases (handoff, "What happens next").
- Web carrier answered (fvs-01): on excel-graph the overflowing `FVSCHEDULE(5916,{-1E+300,1E+300})` reads inside the
  sheet as a live number 0 (TYPE 1, ISNUMBER TRUE, `B1+1` is 1, `"["&B1&"]"` is "[0]"), except `B1/(2^1023)`, which is
  `#NUM!`; ours makes B1 `#DIV/0!` and its observers propagate it (B2, B11, B15 and B22 are right; B4 16, B13 -17,
  B14 2) (financial.md, "FVSCHEDULE whose running product overflows"). The desktop live
  and seeded steps and the isolated fvs-02 variants are still open (T-4250).
- fvs-02: overflow is a live 0 in some calls and `#NUM!` in others (P!B23, B26, B30, B32, B34 `#NUM!`; B4, B5, B24,
  B27, B29, B31, B35 0); a subnormal running product is 0 (B38:B42, settled by E38:E42 and G40). The one-factor cut:
  B10:B18 are a live 0 and B19:B22 DBL_MAX at the sampled k, the cut in (33562623, 33562624] among sampled points,
  where `*` turns from `#NUM!` to DBL_MAX (H18, H19); B3 and B7 are the exact 2x = ±DBL_MAX where `(2^1023-2^970)*2`
  is `#NUM!`.
- Unconfirmed: fvs-02 was filled as one sheet although its protocol allows that only once FVS-01 has shown no carried
  state, and FVS-01's carried-state steps were not run (desktop only). Treat the 0 against `#NUM!` splits (B23/B24,
  B29/B30) as possibly carried state until the per-row isolated runs (T-4250) agree. Ours: `#DIV/0!` for most overflowing calls, but B29 and
  B35 `#NUM!`, B30 0, B34 `#VALUE!` and B14:B18 DBL_MAX (financial.md lists them).
  Column H: `*` near DBL_MAX is `#NUM!` where binary64 gives DBL_MAX (H3, H7) and finite where binary64 overflows
  (H10:H17). Column G (k = -1023) carries no information: 2^-1023 is read as 0. See financial.md "FVSCHEDULE whose
  running product overflows or underflows" and numbers.md "A product at the edge of the double range".
- `B1/(2^1023)` of fvs-01 is `#NUM!`; add a `=0/(2^1023)` control before reading anything into it.
- Settled here (VERIFIED, financial.md "FVSCHEDULE rounds each step twice"): inside the double range each step is the twice-rounded
  product (equal to `*`); near DBL_MAX FVSCHEDULE and `*` part ways (B10:B17 against H10:H17); the
  fvs-02 one-factor cut is where that product stops overflowing. Implementing it (`arithmetic::extended_product` in
  `fn_fvschedule`) is an implementation task to file.
- fvs-03: the MOD fingerprint columns are all `#NUM!` (60 cells); read the raw C, D and E values, which are equal to the
  bit in Excel in rows 12 to 23. Ours differs there (the C-D and C-E columns are ±0.03125 to ±0.125, and C12:C23,
  B3, B7, D3, D7 differ in the low bits).
- When the financial.md entry is settled and renamed, move its old `<a id>` anchor and `FVSCHEDULE_OVERFLOW_SOURCE`
  (tools/src/rustsheet_tools/accuracy/defs/cashflow_values.py) together, then regenerate the affected math sidecars.
- The FORMULATEXT cell of fvs-01 is a separate display issue, owned by T-5111.
- Code: `fn_fvschedule` (functions/financial).

- val3 batch, docs/research/probe-handoff-2026-11-09-val3.md (excel-graph fill, 2026-11-09). Owns: the eight isolated FVS cases probes/val3-fvs-01 to probes/val3-fvs-08.
  Three stay pending: in probes/val3-fvs-05, probes/val3-fvs-06 and probes/val3-fvs-08 (each holding one overflowing
  FVSCHEDULE) the oracle's own canary sheet reads `SQRT(2)` as `#NUM!`, twice in a row, so `rustsheet-oracle fill`
  refuses the workbook; probes/val3-fvs-07, whose schedule zeroes the product before it overflows, filled normally.
  One hypothesis, not yet tested: an overflow leaves state carried across the workbook (the val2-fvs-02 caveat), or it
  is a Graph session effect; decide which before treating any one-sheet FVSCHEDULE result as per-call. No log of the
  refused attempts was kept (2026-11-09, between the fvs-04 and fvs-07 fills, then one retry); rerun to capture one.
- val4 batch, docs/research/probe-request-val4-fvs.md (excel-graph fill, 2026-11-10, one session per case). Owns:
  probes/val4-fvs-01, probes/val4-fvs-02, probes/val4-fvs-03, probes/val4-fvs-04, probes/val4-fvs-05 and
  probes/val4-fvs-06 (every visibly-off cell). No fill was refused. In probes/val4-fvs-03,
  probes/val4-fvs-04 and probes/val4-fvs-05, as the request predicted, Probe!A1 `=2^1023` is `#NUM!` and B1:B5 are 0, 1,
  TRUE, 0, 0. The val4-fvs-06 control (no overflow) has every detector 2^1023. So the overflowing call leaves a state
  that the next calculation of the right kind takes once. Which kind that is, is not the request's "two literal
  operands": in val4-fvs-01 `=2^K1`, `=K4^K1`, `=SQRT(2)`, `=EXP(1)` and `=1/3` take it, while `=1.5^2` leaves it.
  financial.md, "FVSCHEDULE whose running product overflows or underflows", lists every candidate and reader. Score
  those against the request's hypotheses.

## Acceptance criteria
- [ ] Every hypothesis of the family in probe-requests-2026-11-08.md has a verdict (confirmed, refuted, undecided) citing
      the probe cells, written to the research note of the family.
- [ ] Each confirmed rule is a docs/excel-semantics entry citing its probe cells; each has an implementation task filed.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Engine changes (the filed implementation tasks carry them).
- Desktop-only questions (M14, decision 0039).
