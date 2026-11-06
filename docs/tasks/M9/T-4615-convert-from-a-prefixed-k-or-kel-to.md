---
id: T-4615
title: "CONVERT from a micro-prefixed K or kel to Reau: probe and match Excel, which drops the prefix"
milestone: M9
lane: functions-engineering
status: done
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
CONVERT from a prefixed Kelvin (`uK`, `ukel`, ...) to Réaumur gives Excel's value, and the rule is known for every
prefix and every temperature target.

## Context
- The CONVERT Gate A table (T-4085) has two points where Excel ignores a micro prefix on the source when the target is
  `Reau`: `CONVERT(-56.8944444765809,"ukel","Reau")` is -264.03555558126476 and
  `CONVERT(-68268277991730.8,"uK","Reau")` is -54614622393603.16, which are `(v - 273.15)*0.8` of the unprefixed value
  (accuracy/CONVERT/s1 A65, A81, excel-graph). Ours, and the math, scale by 1E-6 first (-218.52..., -54614840.9...).
  engineering.md "CONVERT from a micro-prefixed Kelvin to Réaumur drops the prefix (T-4085)".
- The same table applies a prefix to C and F as the math does (`Ykel`, `Zkel` to `C`, `F` to `hkel`), and
  probes/convert-temperature-prefix-a/-b show the prefix applies between K and kel. Prefixes against `Rank`, a prefixed
  target `Reau` side (`Reau` to `uK`), and other prefixes to `Reau` are unprobed.
- Gate A does not fail on these points (the engine is inside the envelope through the math); Excel stays the target
  (decision 0300), so the engine should match Excel once the rule is probed.
- Code: `convert_temperature` and the temperature branch of `convert_of` in functions/engineering/convert.rs.

## Acceptance criteria
- [x] A probe case covers every decimal prefix on `K` and `kel` against `C`, `F`, `Rank` and `Reau`, in both
  directions, with oracle values; the semantics entry states the rule the cells show.
- [x] The engine matches every probed cell; the CONVERT Gate A table points A65 and A81 are bit-exact.
- [x] `make check` green; reviewers approve.

## Out of scope
- Non-temperature CONVERT last digits (T-2046, T-2047).

## Outcome
- Probe probes/t4615-convert-prefixed-kelvin (346 cells, excel-graph 2026-11-06): the 21 decimal prefixes on `K` and
  `kel` against `C`, `F`, `Rank` and `Reau`, both directions, at 1000; row 23 repeats the Gate A cells A65 and A81 and
  adds a few other values.
- Rule: Excel ignores a Kelvin prefix from K to Rank, from K to Reau and from Reau to K. It applies the prefix from
  Rank to K and against C and F, both ways. The task assumed only `u` toward `Reau`; every prefix is dropped, and also
  toward `Rank` and from `Reau`. The engine already matched Rank to K and the C/F cells (213 of 346 before the fix).
  engineering.md "CONVERT drops a Kelvin prefix toward Rank and Reau" replaces the T-4085 entry.
- Engine: `convert_of`'s temperature branch drops the prefix in those three directions. Unit test
  `kelvin_prefix_is_dropped_toward_rank_and_reau`.
- Eval delta: +1 case, 346/346 cells passing (133 would fail without the fix); 0 regressions. The Gate A points
  accuracy/CONVERT/s1 A65 and A81 now return Excel's values (the probe's S!A23 and S!B23 are the same calls, bit-exact);
  accuracy 0 failing.
- No fit: the rule is structural and holds for every probed cell with no free constant.
