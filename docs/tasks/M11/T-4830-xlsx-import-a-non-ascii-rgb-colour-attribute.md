---
id: T-4830
title: "xlsx import: a non-ASCII rgb colour attribute must not panic the importer (byte slice at 2)"
milestone: M11
lane: engine
status: done
priority: P1
size: S
tier: opus
depends_on: []
---

## Goal
No colour value in a workbook can panic the importer (CLAUDE.md directive 5). An 8-byte `rgb`
attribute, or a theme colour's `val` or `lastClr`, that is not ASCII is kept as written instead of
being cut at byte 2.

## Context
`get_color_indexed` (`import/util.rs`) dropped an ARGB value's alpha byte with `raw[2..]` whenever
`raw.len() == 8`. That is a byte length, so `rgb="Aé12345"` (8 bytes, with byte 2 inside the `é`)
panicked with "start byte index 2 is not a char boundary". The same `raw.len() == 8` and `[2..]`
was in the theme loader's `format_hex` (`import/theme.rs`). The file-wide
`#![allow(clippy::unwrap_used)]` on `util.rs` also hid three `unwrap`s after `has_attribute`
checks. `parse_indexed_colors` (`import/styles.rs`) already guarded its copy of the slice with
`is_ascii`.

## Acceptance criteria
- [x] One helper, `util::strip_alpha`, drops the alpha byte of an 8-digit ASCII ARGB value and
      keeps anything else as written. `get_color_indexed`, `theme::format_hex` and
      `parse_indexed_colors` all use it.
- [x] `util.rs` has no file-wide `unwrap_used` allow. Its `rgb`, `indexed` and `theme` attributes
      are read with `if let`, and the test module keeps its own allow.
- [x] Tests (`rgb_colours_are_never_cut_inside_a_character`, `hex_values_are_never_cut_inside_a_character`)
      panicked before the fix and pass after it. ASCII ARGB and RGB values are unchanged.
- [x] `make check` green; reviewers approve.

## Out of scope
- What Excel does with a non-ASCII colour value. No probe covers it: Excel likely refuses such a
  file, and the importer only has to not panic. No Excel quirk is recorded.

## Outcome
`strip_alpha` is the one definition of the alpha cut. A non-ASCII 8-byte value now becomes
`#` plus the value, uppercased as ASCII, as any other unrecognised value already did. A scan of
`crates/rustsheet-xlsx/src` found no other fixed byte-position slices of text, only fixed-length
hash byte arrays in `encryption.rs`.
