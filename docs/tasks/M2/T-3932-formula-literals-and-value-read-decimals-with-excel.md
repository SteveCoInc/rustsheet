---
id: T-3932
title: "Formula literals and VALUE read decimals with Excel's x87 multiply by 10^-n"
milestone: M2
lane: engine
status: todo
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
- [ ] Every literal (Hits!C) and VALUE (Hits!E) cell of probes-numeric/t3658-round-scaling, and columns C and D of t3658-round-scaling-fresh, pass, with a probe first for positive exponents and longer literals if the change reaches them.
- [ ] No regression in the literal-parsing probes (numbers.md "15-digit literal parsing") or gen suites.
- [ ] Decision 0188 R4 guards for the structural model: a hold-out split (manual ~30% until T-3360) of the model against the correctly rounded conversion, the fresh sweep probes-numeric/t3658-round-scaling-fresh, and the Gate A envelope result, recorded in the Outcome.
- [ ] `make check` green; reviewers approve.

## Out of scope
- ROUND and ROUNDDOWN (T-3931).
- The xlsx stored-number reader (Hits!G): its rule is open.
