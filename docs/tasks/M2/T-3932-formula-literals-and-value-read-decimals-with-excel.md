---
id: T-3932
title: "Formula literals and VALUE read decimals with Excel's x87 multiply by 10^-n"
milestone: M2
lane: engine
status: done
priority: P3
size: M
tier: opus
depends_on: [T-3658]
---

## Goal
Formula number literals and VALUE of numeric text produce the double Excel produces at every literal and VALUE cell of probes-numeric/t3658-round-scaling.

## Context
- numbers.md "Decimals reach a double through an x87 multiply by 10^-n" (T-3658): a literal `mE-n` (digits as
  written, trailing zeros included) is round53(round64(m x round64(10^-n))); VALUE of the same text matches. The
  engine parses to the nearest double.
- Literal parsing is shared with "15-digit literal parsing" (the 15-digit cut comes first); check how the cut and
  this conversion compose for longer literals with a probe before generalizing beyond the probed range
  (m < 1,000,000, n = 0..15, negative exponents only).
- Failing cells: probes-numeric/t3658-round-scaling Hits!C1:C6, Hits!C8:C16, Hits!C19:C20, Hits!C22:C23, Hits!C25:C47, Hits!C52:C61, Hits!E1:E6, Hits!E8:E16, Hits!E19:E20, Hits!E22:E23, Hits!E25:E47, Hits!E52:E61
- Fresh sweep: probes-numeric/t3658-round-scaling-fresh Calc!C1:C152, Calc!C156, Calc!C161:C162, Calc!C164:C165, Calc!C167:C169, Calc!C173, Calc!C175, Calc!C177:C179, Calc!C181, Calc!D1:D152, Calc!D156, Calc!D161:D162, Calc!D164:D165, Calc!D167:D169, Calc!D173, Calc!D175, Calc!D177:D179, Calc!D181
- 15-digit literal and complex-text point (T-3945): probes/t3945-atan-near-midpoint S!B203:B204, S!C1, S!C3, S!C5,
  S!C7:C10 (`0.190581435720431` reads one ulp above the nearest double as a literal, through VALUE and through the
  complex functions' text parse), and probes-numeric/t3643-r4-log-modulus C111, which is the same ulp reaching
  IMLOG2 through COMPLEX's text (engineering.md "IMLOG10 and IMLOG2 scale IMLN").
  T-3997 (M5) also lists S!C8 and S!C10 as their M5 exit owner and checks them once this task lands.
- Stored xlsx numbers do not follow this model (numbers.md "Stored numbers in an xlsx are not always parsed to the
  nearest double", OPEN); their off cells are listed here so they have an owner, but changing the xlsx reader is
  out of scope until that rule is derived: probes-numeric/t3658-round-scaling Hits!G1:G6, Hits!G8, Hits!G10:G11, Hits!G14:G16, Hits!G18:G19, Hits!G22:G26, Hits!G29:G41, Hits!G45:G47, Hits!G49:G58, Hits!G60:G61

## Acceptance criteria
- [x] Every literal (Hits!C) and VALUE (Hits!E) cell of probes-numeric/t3658-round-scaling, and columns C and D of t3658-round-scaling-fresh, pass, with a probe first for positive exponents and longer literals if the change reaches them.
- [x] No regression in the literal-parsing probes (numbers.md "15-digit literal parsing") or gen suites.
- [x] Decision 0188 R4 guards for the structural model: a hold-out split (manual ~30% until T-3360) of the model against the correctly rounded conversion, the fresh sweep probes-numeric/t3658-round-scaling-fresh, and the Gate A envelope result, recorded in the Outcome.
- [x] `make check` green; reviewers approve.

## Out of scope
- ROUND and ROUNDDOWN (T-3931).
- The xlsx stored-number reader (Hits!G): its rule is open.

## Outcome
- `parse_excel_number` (formula literals, VALUE and the other number-text readers) converts number text whose
  significant digits as written are below 1,000,000, with n = 1..15 and no positive exponent, through
  `decimal_round::x87_decimal_to_f64` (T-3931's helper, now `pub(crate)`): the digits as written, trailing
  zeros included, times round64(10^-n), rounded to 64 bits and stored. Longer text, positive exponents and
  n > 15 keep the nearest double, so no probe was needed for them. The 15-digit T-3945 point
  (probes/t3945-atan-near-midpoint S!B203:B204, S!C1, S!C3, S!C5, S!C7:C10, probes-numeric/t3643-r4-log-modulus
  C111) therefore still fails, as in the baseline: T-4077 owns those cells and the longer-literal probe, and
  T-3997 now depends on it.
- Formulas are reparsed from their printed text, and a literal printed at 15 digits can read one ulp off
  (`=4910E-9` as `4.91e-6` reads one ulp high, `=14550E-15` as `1.455e-11` reads as the nearest):
  `literal_text` in stringify/literals.rs prints the digits with trailing zeros added (`4910e-9`, `14550e-15`)
  or, failing that, all 15 digits, whichever reads back to the same double.
- Eval delta: probes-numeric/t3658-round-scaling 2,388 → 2,492 of 2,539 (+104); the 47 left are the stored xlsx
  numbers (Hits!G, `=F#`, out of scope). t3658-round-scaling-fresh 392 → 724 of 724 (+332, all pass). Gated
  suites: fixed 0, regressions 0 (probes-numeric is ungated); Gate A unchanged.
- Readers that share `parse_excel_number` but were not probed (typed input and percent text, `"text"+0`,
  criteria, NUMBERVALUE, AVERAGE text, DATEDIF) and the printed literal text are recorded UNVERIFIED in
  numbers.md, with the probe task T-4074.
- Decision 0188 R4 (structural model, kind (c)). Hold-out: the manual split T-3931 fixed (sha1(`case/Sheet!Ref`)
  first byte below 77, ~30%), margin 2 points, over the pure literal and `VALUE("…")` cells. Bit-exact counts
  (model / correctly rounded):
  | case, reader | train | hold-out |
  |---|---|---|
  | t3658-round-scaling literal | 77/77 / 43 | 45/45 / 27 |
  | t3658-round-scaling VALUE | 42/42 / 5 | 19/19 / 4 |
  | fresh literal | 114/114 / 10 | 67/67 / 5 |
  | fresh VALUE | 113/113 / 7 | 68/68 / 8 |
  The model column is the engine's own result (`rustsheet-eval show`: every literal and VALUE cell of both cases
  passes). Both candidates hold every cell within 4 ulp and within 15 digits; the model's hold-out rate (100%)
  equals its train rate. Fresh sweep: probes-numeric/t3658-round-scaling-fresh (frozen model, T-3658) 362/362 literal and
  VALUE cells bit-exact, so all inside the Gate A envelope.
- Follow-ups: T-4063 (probe TRUNC and ROUND with nothing dropped at these points, from the T-3931 fidelity review);
  T-4074 (probe the unprobed readers and FORMULATEXT, from this task's fidelity review); T-4077 (7- to 15-digit
  literals and number text, the cells above).
