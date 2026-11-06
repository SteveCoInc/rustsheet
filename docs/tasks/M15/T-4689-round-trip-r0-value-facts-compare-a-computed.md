---
id: T-4689
title: "Round-trip R0 value facts compare a computed value as the export saves it: -0 as 0, a kept positive subnormal as 0, the engine-only #ERROR! as #VALUE! (3,107 false changed items)"
milestone: M15
lane: harness
status: blocked
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
The round-trip suite's R0 semantic items stop reporting `changed` for a cached value that the export writes
exactly as the documented save rules say. A value item then fails only when a save really loses or alters a
value.

## Context
- Gap triage 2026-11-06 (`make eval` on origin/main ebe81baf6, calipers, corpus-bench and poi fetched). In
  eval/runs/latest/roundtrip.json, 3,107 R0 `semantic` items with verdict `changed` fall in three groups. In each
  group the save follows a rule the export already documents:
  - 3,080 items in 43 calipers pv goldens: `Number(-0.0) became Number(0.0)`. Example: roundtrip-calipers
    pv_goldens/pv23_006/golden.xlsx, `value CASHFLOW!C28` (`=-FUNDING!C49`, cached `<v>0</v>` in the source).
    pv26_018 and pv26_2_018 have 540 each, pv26_016 and pv26_2_016 412 each. The engine computes -0.0 there. The
    sign of zero cannot be observed in Excel (docs/excel-semantics/coercion.md: "`(0)` and `($0)` are 0 (the sign
    of zero is not observable)"; criteria.md: "-0.0 equals 0.0").
  - 8 items in 4 calipers cases, formulas_datetime and formulas_math_basic (golden.xlsx and init.xlsx): `Number(6e-323)
    became Number(0.0)`. Example: `value Math Tests!H24` (`=$C$11+$C$10`, cached `<v>0</v>`). The export saves a
    positive kept subnormal as 0, as Excel for the web does (`saved_number` in
    crates/rustsheet-xlsx/src/export/worksheets.rs, T-2211, probes/subnormal-saved-as-zero, decision 0089).
  - 19 items in 3 cases: `Error(Unknown) became Error(Value)`. They are roundtrip-poi/clone_sheet.xlsx (16,
    Arkusz1!M10:M25, `L10-(-4,3)`), roundtrip-poi/bug67784.xlsx (2, Sheet1!A7:A8, a three-argument FLOOR) and
    roundtrip-corpus-bench/248-48/1_248-48_input.xlsx (1, `Ref Program Names (3)!A3`, `COUNTA([1]!#REF!)`). The engine
    gives its own `#ERROR!` (`ErrorKind::Unknown`), and the export saves engine-only errors as `#VALUE!`
    (`exported_error_literal`, T-1133, docs/excel-semantics/syntax.md).
- Why it happens: `value_facts` in crates/rustsheet-eval/src/roundtrip/semantic.rs prints each value with `{:?}` (bits)
  and compares the recalculated workbook that was saved against the output as loaded (T-4513). The save rules above
  are not applied to the source side, so every value they change is reported. SPEC-0033 D3 rule 1 says "A value
  compares by type and bits".
- Before the fix, find out where the sign of -0.0 is lost. The export writes `format!("{v}")` after `saved_number`,
  which keeps -0.0, so either `<v>-0</v>` is read back as 0.0 or the zero is normalised before that. Record which.
- One definition per concept: `saved_number` and `exported_error_literal` are private to rustsheet-xlsx. Expose them
  (or one helper "the value as saved") and call that helper. Do not copy the rules into the harness.
- The items are case-level known failures in eval/baseline/roundtrip-calipers.txt, roundtrip-corpus-bench.txt and
  roundtrip-poi.txt (`F<TAB>case<TAB>* <count> <hash>`).

## Acceptance criteria
- [ ] `value_facts` (or the `compare` step for value items) reads the source side through the export's own save
      rules, so -0.0 compares equal to 0.0, a positive subnormal to 0, and `#ERROR!`, `#N/IMPL!` and `#CIRC!` to
      `#VALUE!`. Unit tests in crates/rustsheet-eval/src/roundtrip/tests.rs cover each of the three, plus one real
      change that is still reported (1 became 2).
- [ ] SPEC-0033 D3 rule 1 says the source side of a value item is compared in its saved form, and cites the save
      rules.
- [ ] After `rustsheet-eval roundtrip --update-baseline` with every suite fetched, eval/runs/latest/roundtrip.json
      has no R0 `changed` value item of these three kinds. The case counts of the cases above fall by those items,
      and the baselines gain no new F line.
- [ ] The Outcome records where the sign of -0.0 was lost (export or import).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Whether the engine should compute -0.0 at all. It cannot be observed in a cell's value, and the function suites
  score that.
- The 42 `lost` items of calipers roundtrip/formulas_information (golden.xlsx and init.xlsx, Sheet1!O140:O160 in
  each). They are the spill half of desktop Excel's `CELL("width")` result, which the engine does not give
  (decision 0308 item 4, T-4644).
- Package-validator items (T-4613, T-4690).

## Blocked because
Deferred to Phase 2 (decisions 0027, 0275, 0306): M15 waits until Phase 1 parity exits, except for the task the
owner-directed session claims. Unblock when docs/status.md has the line `**Phase 1:** exited <date>`, or by claiming
it in that session.
