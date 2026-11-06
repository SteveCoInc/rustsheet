---
id: T-4599
title: "Reword text.md's TEXTBEFORE UTF-16 sentence and give the text functions one UTF-16 length helper (T-4474 review follow-up)"
milestone: M7
lane: functions-text
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
text.md's TEXTBEFORE/TEXTAFTER UTF-16 sentence reads clearly, and the text functions count UTF-16 units through one
shared helper instead of repeating `encode_utf16().count()`.

## Context
- Review of T-4474 (#3602), all MINOR:
  - excel-fidelity: in docs/excel-semantics/text.md "TEXTBEFORE and TEXTAFTER: instance_num past the text, a number
    if_not_found", the sentence "measure the text and each delimiter in UTF-16 units of the delimiter as written" is
    garbled. Reword it as "measure the text and each delimiter in UTF-16 units, the delimiter as written (never its
    case fold)".
  - design: `units` in crates/rustsheet-engine/src/functions/text/before_after.rs repeats the
    `s.encode_utf16().count()` idiom of text/concatenate.rs, text/util.rs and text/common.rs. Move one helper into the
    shared text module (`functions/text/util.rs` or `common.rs`) and use it in those files. No behaviour change.
- The comment above the every-delimiter-longer check still says "UNVERIFIED for a list". probes/t4474-textbefore-
  utf16-bounds B6 covers a list of equal-length entries only. Keep the word, but cite B6.

## Acceptance criteria
- [ ] The text.md sentence is reworded as above.
- [ ] One UTF-16 length helper serves before_after.rs, concatenate.rs, util.rs and common.rs (`rg "encode_utf16\(\)\.count" crates/rustsheet-engine/src/functions/text` lists only the helper).
- [ ] No eval change; `make check` green; reviewers approve.

## Out of scope
- Any TEXTBEFORE/TEXTAFTER behaviour change.
