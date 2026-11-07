---
id: T-4777
title: "Math sidecars for a call whose argument is another call's formula cell (GAMMA.DIST at GAMMA.INV's quantile, BINOM.DIST at BINOM.INV's count): 177 counted cells"
milestone: M9
lane: harness
status: todo
priority: P1
size: M
tier: opus
depends_on: []
---

## Goal
A one-call cell whose argument reads a formula cell holding a Gate A call gets a math value under a rule that a decision
record states (an amendment of decision 0297 or a new record). The cells are of the form `=GAMMA.DIST(A14-4*M14,2.5,1,TRUE)`
with `A14` `=GAMMA.INV(1E-200,2.5,1)`, `=BINOM.DIST(A36,100000,0.001,TRUE)` with `A36` `=BINOM.INV(100000,0.001,0.3)`, and
`=BETA.INV(Z1,3,600)` with `Z1` `=BETA.DIST(...)`. Then the 177 visibly-off cells below, which decision 0188 R1(c) counts
today only because they have no math value (decision 0325), are sorted by the math: Phase 1b evidence, or a `campaign:`
cell for their owner.

## Context
- Gap triage 2026-11-07 (`make eval` at b9ae5216f, excel-graph). These are the largest group of counted M9 probe cells
  with no math value. `rustsheet_tools.accuracy.eval_math.candidates` gives each one `unparsed (UnparsedError)`. The
  module doc (SPEC-0029 "Math references for eval cells") says why: "an argument reads a formula cell (its value is the
  engine's or Excel's, not a literal)". Only exact-input arithmetic cells are read. Nested calls in one formula are
  already composites (T-3993, decision 0297 option (b): inner calls at their correctly rounded math doubles).
- Why this is a design question (opus), not the mechanical "treat a reference as a nested call": the probes were built
  to evaluate the outer call at Excel's own inner result. probes-numeric/t3311-gamma-inv-lower-tail-cdf (T-3311) reads
  GAMMA.DIST and CHISQ.DIST at Excel's GAMMA.INV/CHISQ.INV quantile A and at A ± 1 to 4 ulp (M = the ulp), to see where
  Excel's search lands. Excel's quantile is often not the correctly rounded one, so option (b)'s inner double (the
  math quantile) puts the cdf at a different point than Excel's. Excel's B14 would then look off the math when its
  GAMMA.DIST is exact. The options the record should weigh:
  (a) the math at the referenced cell's Excel value (the oracle sidecar's double; the sidecar then depends on the fill,
      so its fingerprint must cover the oracle value it read);
  (b) decision 0297's composite rule, with the reference inlined as a nested call;
  (c) a value only when the engine's and Excel's inner doubles agree, and skip otherwise (`inputs differ`).
  R1(c) reads the engine's result against this one math value, and the engine evaluates at its own inner double, so the
  rule must also say what happens when the two inner doubles differ.
- Inner cells the rule must cover: t3311's M column is `=POWER(2,INT(LOG(A1,2))-52)`, an exact power of two when it is
  read at a double A. An exact-scale reader exists for `*2^k` (T-3924), but not for POWER/INT/LOG of a call result.
- Code: `tools/src/rustsheet_tools/accuracy/eval_math.py` (`LiteralBook`, `candidates`), `composite.py`, `fingerprint.py`;
  the Rust reader that attaches `math` only while `f` matches the formula (crates/rustsheet-eval, `accuracy`).
- Owners of the cells' fix work, which keep them: T-3403 and T-3412 (t3311, live) and deferred T-3565, T-3659, T-3804,
  T-3939 (t3311), T-3562 (t3566). The t3444 Inv cells are T-4773's (filed in parallel), which attaches their math; the rule here must agree with it.
  The triage's mpmath check of those 15 cells (the cdf at Excel's count, a 60-digit sum of the pmf terms) found
  Excel 2.5e-13 to 1.3e-12 off the math and ours 4e-16 to 4.5e-13 off, so they would be Phase 1b evidence under option (a)
  or (b). For these 15 cells options (a) and (b) agree, because the sidecar's BINOM.INV math equals Excel's count.

Cells this task owns for their math value:
- probes-numeric/t3311-gamma-inv-lower-tail-cdf.case.json (174): S!B1, S!B4, S!B5, S!B10, S!B11, S!B12, S!B14, S!C1, S!C4, S!C5, S!C10, S!C11, S!C12, S!C14, S!D1, S!D4, S!D5, S!D10, S!D12, S!D14, S!E1, S!E4, S!E5, S!E10, S!E12, S!E14, S!F1, S!F4, S!F5, S!F10, S!F14, S!G1, S!G5, S!G10, S!G12, S!G14, S!H1, S!H10, S!H11, S!H12, S!H14, S!I1, S!I4, S!I9, S!I10, S!I11, S!I12, S!I14, S!J1, S!J4, S!J9, S!J10, S!J11, S!J12, S!J14, S!K3, S!K4, S!K7, S!K9, S!K15, S!L1, S!L4, S!L5, S!L7, S!L8, S!L10, S!L14, W!C4, W!C6, W!C11, W!C20, W!C21, W!C22, W!C23, W!C26, W!C38, W!C40, W!C44, W!C46, W!C48, W!C51, W!C53, W!C54, W!C56, W!C62, W!C64, W!C67, W!C72, W!C73, W!C75, W!C76, W!C78, W!C81, W!C83, W!C85, W!C86, W!C88, W!D4, W!D6, W!D11, W!D17, W!D20, W!D21, W!D22, W!D27, W!D28, W!D35, W!D38, W!D40, W!D48, W!D53, W!D54, W!D56, W!D59, W!D61, W!D62, W!D64, W!D67, W!D68, W!D72, W!D75, W!D76, W!D78, W!D86, W!D87, W!D88, W!E4, W!E6, W!E11, W!E17, W!E20, W!E21, W!E22, W!E24, W!E27, W!E28, W!E35, W!E40, W!E53, W!E56, W!E61, W!E62, W!E64, W!E70, W!E72, W!F6, W!F30, W!F31, W!F37, W!F38, W!F40, W!F46, W!F48, W!F54, W!F56, W!F61, W!F62, W!F63, W!F64, W!F68, W!F69, W!F70, W!F71, W!F72, W!F74, W!F76, W!F78, W!F80, W!F81, W!F82, W!F85, W!F86, W!F87, W!F88
- probes-numeric/t3566-beta-inv-tiny-shape-and-range.case.json (3): Calc!A229, Calc!A231, Calc!A233

## Acceptance criteria
- [ ] A decision record (`make new-decision`) chooses among (a), (b) and (c) or another rule, with a worked example from
      t3311 (one cell where Excel's quantile is the correctly rounded one, and one where it is not). SPEC-0029's
      eval-cell list and the `eval_math` module doc state the rule.
- [ ] `rustsheet-accuracy math --suite probes` and `--suite probes-numeric` give the cells above a value, or a listed
      skip reason the record names (for example `inputs differ`). `rustsheet-accuracy math --check` passes. If the
      value reads the oracle, the sidecar fingerprint changes when the referenced oracle value changes, and a test
      shows it.
- [ ] After `make eval`, the Outcome gives the counts: how many of the 177 cells are Phase 1b evidence, how many still
      count (and why), and the M9 probe row's R1(c) count before and after. `eval/baseline/` gains no new line.
- [ ] `make check` green; reviewers approve (excel-fidelity included).

## Out of scope
- Engine changes to GAMMA.DIST, CHISQ.DIST, BINOM.DIST or BETA.INV (their owners above).
- References to cells outside the case's own sheets, and calls with no Gate A definition.
