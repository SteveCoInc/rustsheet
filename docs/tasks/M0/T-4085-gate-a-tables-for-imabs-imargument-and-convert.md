---
id: T-4085
title: "Gate A tables for IMABS, IMARGUMENT and CONVERT, replacing their composite exemptions (after T-4084)"
milestone: M0
lane: harness
status: done
priority: P3
size: M
tier: opus
depends_on: [T-4084]
spec: docs/specs/SPEC-0029-gate-a-accuracy-gate.md
---

## Goal
IMABS, IMARGUMENT and CONVERT get Gate A tables, and their `composite` lines leave `eval/accuracy/exempt.tsv`
(decision 0270).

## Context
- Decision 0270 (T-3800) made them `composite` until the generator has a text kind (T-4084).
- Kernels and fitted rules: engineering.md "IM* reciprocal kernels, IMABS and IMARGUMENT (T-2044)", "CONVERT constants
  and evaluation order", "CONVERT temperature", "CONVERT unit names". Open last-digit work: T-2538 (IMABS and
  IMARGUMENT), T-2046 and T-2047 (CONVERT).
- Removing an `exempt.tsv` line is a protected-path change: it needs a decision record.

## Acceptance criteria
- [x] One cited mpmath definition per function, with unit tests; committed, oracle-filled `s1` tables.
- [x] The three `exempt.tsv` lines are removed under a decision record. `missing tables` does not rise.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Other IM* functions, which return complex text, not a number.

## Outcome
- `tools/src/rustsheet_tools/accuracy/defs/complex_modulus.py`: IMABS (|z|) and IMARGUMENT (atan2 on (-π, π]) over T-4084's
  complex text kind, with a custom draw (centre, near-axis tail, the grammar's edge texts with zero for a quarter of
  them, extreme and log-uniform parts).
  - Domain error: `IMARGUMENT` of 0 is `#DIV/0!` (im-last-digit-sweep C21). A quotient y/x past the double range has no
    value (NoValueError): ATAN2's `#NUM!` there is unprobed for IMARGUMENT.
- `tools/src/rustsheet_tools/accuracy/defs/convert.py`: CONVERT over 1,259 unit strings (every unit name and every
  prefix on every unit that takes it, engineering.md "CONVERT unit names"), read within the other unit's class:
  1,257 evidenced by oracle cells plus the bare `cel` and `fah`, which no oracle cell has yet (no table point
  draws them).
  - The math is exact with each unit's definition (or Excel's documented double: `sg`, `parsec`, `eV`, `u`, `mmHg`), the
    prefixes as the documented doubles (one combined constant; each unit's own with a binary prefix) and the
    temperature forms with their constants as doubles. Units of two classes are `#N/A`.
  - No value where `value * F_from` or the quotient before the prefixes is a nonzero below the smallest normal, or the
    quotient overflows (unprobed).
- Pre-generation check over the literal one-call cells of eval/suites: CONVERT 2,746 number/number and 58
  `#N/A`/`#N/A`; IMABS 269 and IMARGUMENT 237 number/number, 2 `#DIV/0!`/`#DIV/0!`. No math error against an Excel
  number and no math number against an Excel error. One cell, `CONVERT(2.2250738585072E-308,"kg","kg")` (a subnormal
  literal, Excel 0), has no value. The check found the `flb` factor 100 times too small, fixed before generating.
- Tests: tools/tests/test_accuracy_im_convert.py (documented Excel values, domains, class reading, draws).
- Tables `eval/accuracy/tables/{IMABS,IMARGUMENT,CONVERT}/s1`: 120, 120 and 200 points (4 and 12 domain errors), filled
  from excel-graph with 0 failures. Decision 0319 removes the three `composite` lines from `exempt.tsv`.
- `rustsheet-eval accuracy`: 440 new points, all inside the envelope (438 on both sides, 2 through the math only).
  0 failing, 0 table defects, 0 known failures added. `missing tables` 25 before and after on the base (21 after merging main, which added tables).
- Engine bit-exact with Excel: IMABS 119/120 (the miss is T-2538's), IMARGUMENT 120/120, CONVERT 188/200 (10 last-digit
  misses inside the envelope on both sides, T-2046/T-2047; 2 are the cells below).
- **Excel kept as the target (decision 0300):** CONVERT s1 A65 `CONVERT(-56.8944444765809,"ukel","Reau")` (Excel
  -264.03555558126476, math -218.52004551555558) and A81 `CONVERT(-68268277991730.8,"uK","Reau")` (Excel
  -54614622393603.16, math -54614840.91338464): Excel drops the micro prefix on a Kelvin source toward Réaumur. New
  quirk in engineering.md "CONVERT from a micro-prefixed Kelvin to Réaumur drops the prefix (T-4085)"; the engine follows the
  math there today, so T-4615 probes the rule and matches Excel. No IMABS or IMARGUMENT point and no eval cell with a
  math sidecar is more than 1E-4 off the math.
- Math sidecars regenerated with `--function` for probes (23 new, 2 extended) and probes-numeric (3 new); additions
  only; `make eval-math-check` current.
- Review follow-up: T-4620 cross-checks `UNITS` and the factors against the engine's table.
- Semantics: engineering.md "Gate A tables for IMABS, IMARGUMENT and CONVERT (T-4085)" and the Réaumur entry.
