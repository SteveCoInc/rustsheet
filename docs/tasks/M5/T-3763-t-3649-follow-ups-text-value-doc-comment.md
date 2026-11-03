---
id: T-3763
title: "T-3649 follow-ups: text-value doc comment, display color of a color-only section, test split"
milestone: M5
lane: functions-text
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
The docs and tests left by T-3649 (PR #2780) say only what the evidence shows, and each rule has its own test.

## Context
These are the MINOR review findings on PR #2780, filed rather than fixed there to keep the reviewed head.
- `format_text_value`'s doc comment (crates/rustsheet-engine/src/formatter/text_value.rs) still says a text value under fewer than four sections is unprobed. probes/text-value-kind-sweep F2:G69 and probes/text-value-kind-sweep-b F2:G20 now cover it, so cite them.
- number-formats.md "A section of only a color or a condition prints General" says "The cell keeps the section's color". The probes test TEXT() strings only and never observe color. Mark that sentence inferred for cell display, or probe the displayed color (desktop question, decision 0039).
- `trailing_separator_and_bracket_only_sections` in formatter/test/test_condition.rs mixes two rules. Split it into one test per rule, and move the text-value assertions next to the other text-value tests. Wrap the long line in that number-formats.md entry.
- The bracket-only arm in `format_number` applies the section's numeral system (`p.numerals`, from the T-3532 merge). No probe covers a color or condition section that also carries a numeral-system tag (`[Red][$-2010000]`). Say so in the entry, or add a probe cell.
- A trailing `;` now adds an empty part, so `parts.last()` for `x@;` / `@;` is that empty section. Cell display of those shapes is unprobed. Add cells for them to a probe (TEXT and display).

## Acceptance criteria
- [x] The doc comment cites the probes; no "unprobed" claim remains for one to three sections.
- [x] The color sentence is marked inferred, or backed by a probe.
- [x] The numeral-tag combination and the `x@;` shapes are probed, or marked unprobed in number-formats.md.
- [x] One test per rule; behavior unchanged (no eval change).
- [x] `make check` green; reviewers approve.

## Out of scope
- Any behavior change.

## Outcome
Doc comment of `format_text_value` cites the sweeps for fewer than four sections; number-formats.md marks the cell color
as inferred and lists the numeral-tag combination and the `x@;` / `@;` shapes as unprobed (no new probe, no oracle
needed); the mixed test is split into `trailing_separator_opens_an_empty_section` and
`bracket_only_section_prints_general`, and the text-value assertions moved to `empty_text_section_is_empty`
(test_locale_tag.rs). No behavior or eval change.
