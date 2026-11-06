---
id: T-4658
title: "Probe how Excel saves a 3-D span over another workbook's sheets: [1]First:Last! or '[1]First:Last'!"
milestone: M11
lane: research
status: todo
priority: P3
size: S
tier: sonnet
depends_on: [T-4140]
---

## Goal
Evidence for the spelling Excel writes in a saved file for a 3-D span over a linked workbook's sheets, so the
engine's export of `[1]Uses:Defines!$B$2` is evidenced instead of extrapolated.

## Context
- T-4140 (PR #3710) prints another workbook's `[n]Sheet` bare unless the sheet part needs quotes, quoted
  whole otherwise. The evidence is Excel-saved corpus files: references.md, "A formula's reference to
  another workbook is written `[1]Sheet!A1`".
- For a span, `[1]Uses:Defines`, it follows the local-span rule (`sheet_span_text`): bare unless either end
  needs quotes. That is UNVERIFIED. The only evidence is fixtures/external-link-cached-3d-span. RustSheet
  built that file, so it shows only that Excel for the web reads the bare form, not what Excel writes.
- Raised by T-4140's excel-fidelity review (MINOR). T-2331 probes how these spans evaluate, not how they
  are spelled.
- Method:
  1. Have Excel save a workbook holding `=SUM([1]Uses:Defines!$B$2)`, and one whose end names need
     quotes (`[1]Uses:Q 4`).
  2. Read the `<f>` text with `rustsheet-eval snapshot`.
  3. If Excel for the web will not save such a workbook, record that and what blocks it (excel-probe skill).

## Acceptance criteria
- [ ] Excel's saved `<f>` spelling for both spans is recorded in references.md, with the probe cited, and the
      UNVERIFIED note in the T-4140 entry is settled. Or the entry records why it cannot be probed.
- [ ] If Excel's form differs, a follow-up task to change `quote_name` is filed.

## Out of scope
- How spans evaluate (T-2331).
