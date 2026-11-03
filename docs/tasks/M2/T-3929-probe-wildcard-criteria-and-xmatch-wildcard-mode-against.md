---
id: T-3929
title: "Probe wildcard criteria and XMATCH wildcard mode against T-3445's lookup fold (weightless, astral, wider classes)"
milestone: M2
lane: engine
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
Oracle evidence settles whether wildcard criteria (COUNTIF(S), SUMIF(S), D-functions) and XMATCH/XLOOKUP's wildcard
mode fold text like exact MATCH (T-3445's `fold_lookup_char`), and the engine and the semantics notes follow it.

## Context
- T-3445 routed `WildcardPattern::new` through `fold_lookup` because exact MATCH's text keys share it. Its evidence
  (probes/t3298-case-mapping-sweep, probes/t3584-match-fold-weightless) is exact MATCH/VLOOKUP/HLOOKUP only, so
  criteria.md "Case beyond ASCII" and lookup.md mark the wildcard paths UNVERIFIED (T-3445 review, excel-fidelity).
- The non-wildcard COUNTIF path treats weightless characters differently from exact MATCH: `"a"&U+0524&"b"` equals
  `"ab"` under COUNTIF but not under MATCH (coercion.md "Weightless characters in `=`, COUNTIF and XMATCH").
- Probe, with excel-probe: COUNTIF/SUMIF/DCOUNTA/XMATCH(..,2) with `"a?b"` against a&U+0524&b, `"a*"` against
  a&U+0524, `"𐐀*"` against 𐐨x, `"ϳ*"`/`"Ͷ*"`, `"ϴ*"` against θ and ϑ, `"Ɒ*"` against ɒ, and the D-function prefix
  criteria (plain text) with the same keys, which keep `fold_case` today.
- Code: `WildcardPattern::new`, `folded_units` and `result_starts_with_units` (functions/criteria.rs).

## Acceptance criteria
- [x] An excel-graph probe case covers the keys above; its findings are in criteria.md and lookup.md (the UNVERIFIED
      notes resolved).
- [x] The engine matches the probe (no regressions).
- [x] `make check` green; reviewers approve.

## Out of scope
- `=` and COUNTIF's non-wildcard weightless equality (T-3904); MATCH's U+0001 key (T-3921).

## Outcome
- Probes (excel-graph, 2026-11-03): probes/t3929-wildcard-fold-criteria (M2: COUNTIF, SUMIF, COUNTIFS, MATCH),
  -xmatch (M6: XMATCH/XLOOKUP match_mode 2), -dfunctions (M9: DCOUNTA wildcard and plain prefix criteria) over 30
  key pairs, and probes/t3929-dfunction-wildcard-prefix (M9, 15 rows) after -dfunctions Calc!B14 showed `"?"`
  counting an astral character.
- Findings: every wildcard path (criteria, MATCH, XMATCH/XLOOKUP wildcard mode, D-functions) folds like exact MATCH
  (`fold_lookup`), and it already did for all but the D-functions. The D-function prefix criteria did not
  (`fold_case`): now `fold_lookup`. New quirk: a D-function wildcard criterion without `=` matches by prefix
  (`"a?"` counts abc), as its plain text does; `WildcardPattern::match_any_tail`.
- criteria.md "Case beyond ASCII" and lookup.md: the T-3445 UNVERIFIED notes resolved; criteria.md "D-function
  plain-text equality matches by prefix" gains the wildcard rule.
- Eval delta: fixed 0, regressions 0; 4 new probe cases (725 cells), 709 pass. The 16 off cells are COUNTIF,
  SUMIF, COUNTIFS and XMATCH(..,0) *without* a wildcard (weightless and T-3584-class equality under the `=`
  collation), added to T-3904.
