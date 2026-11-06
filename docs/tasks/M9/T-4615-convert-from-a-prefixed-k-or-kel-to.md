---
id: T-4615
title: "CONVERT from a prefixed K or kel to Reau: probe and match Excel, which drops the prefix"
milestone: M9
lane: functions-engineering
status: todo
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
  engineering.md "CONVERT from a prefixed Kelvin to Réaumur drops the prefix (T-4085)".
- The same table applies a prefix to C and F as the math does (`Ykel`, `Zkel` to `C`, `F` to `hkel`), and
  probes/convert-temperature-prefix-a/-b show the prefix applies between K and kel. Prefixes against `Rank`, a prefixed
  target `Reau` side (`Reau` to `uK`), and other prefixes to `Reau` are unprobed.
- Gate A does not fail on these points (the engine is inside the envelope through the math); Excel stays the target
  (decision 0300), so the engine should match Excel once the rule is probed.
- Code: `convert_temperature` and the temperature branch of `convert_of` in functions/engineering/convert.rs.

## Acceptance criteria
- [ ] A probe case covers every decimal prefix on `K` and `kel` against `C`, `F`, `Rank` and `Reau`, in both
  directions, with oracle values; the semantics entry states the rule the cells show.
- [ ] The engine matches every probed cell; the CONVERT Gate A table points A65 and A81 are bit-exact.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Non-temperature CONVERT last digits (T-2046, T-2047).
