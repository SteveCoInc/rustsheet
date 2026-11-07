---
id: T-4158
title: "One table for script digit blocks: SCRIPT_DIGITS agrees with Numerals::from_byte, and rename the stale coercion-only test"
milestone: M5
lane: engine
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
The script digit blocks and their numeral bytes have one definition, so the typed-input tags and the formatter's digits cannot drift apart.

## Context
T-3891 (PR #3145) added `SCRIPT_DIGITS` (formatter/parse_formatted.rs: zero code point, locale, numeral suffix `"200"`..`"1200"`)
next to `Numerals::from_byte` (formatter/numerals.rs), which maps the same numeral bytes to the same zero code points
(reviewer-design MINOR, PR #3145). Derive one from the other (for example store the numeral byte in `SCRIPT_DIGITS` and
format the suffix as `{byte:X}00`), or at least add a test that every entry's suffix read by `locale_tag::numerals`
prints its own zero digit. Also rename the test `script_digits_are_mapped_only_by_the_coercion_helper`
(formatter/test/test_parse_formatted_number.rs), which T-3891 made stale: typed input maps them too.

## Acceptance criteria
- [ ] No second hand-written list of the twelve blocks' numeral bytes; a test ties `SCRIPT_DIGITS` to `Numerals`.
- [ ] The stale test name is gone; behaviour unchanged (eval no change).
- [ ] `make check` green; reviewers approve.

## Out of scope
- New blocks or tags (T-4107, T-4156).

## Outcome
`SCRIPT_DIGITS` now stores the numeral byte (suffix formatted as `{byte:X}00` in `script_digit_format`), and a unit test ties every entry to `Numerals::from_byte`. Renamed the stale coercion test to `script_digit_coercion_helper_maps_each_block`. Behaviour unchanged; eval delta none.
