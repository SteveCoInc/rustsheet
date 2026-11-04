---
id: T-4079
title: "Probe: what Excel does with an out-of-range calcPr iterateCount or iterateDelta in a file"
milestone: M12
lane: research
status: todo
priority: P2
size: S
tier: sonnet
depends_on: [T-4048]
---

## Goal
docs/excel-semantics/recalc.md, "Iteration settings outside Excel's range fall back to its
defaults", cites oracle evidence for what Excel does with an out-of-range `calcPr@iterateCount`
or `iterateDelta` in an xlsx. The entry is confirmed or rewritten from that evidence.

## Context
T-4048's Excel-fidelity review (MINOR finding) asked for this. The entry says outright that it
has no evidence for Excel's handling in a file. The 1 to 32,767 range comes from the Options
dialog. The fallback to 100 and 0.001, each setting on its own, is our choice (a directive-5
limit, `Iteration::within_excel_range`), and the xlsx importer has behaved this way since T-0447.

Use the `excel-probe` skill. Write a probe case per setting: a `=A1+1` self-cycle with iteration
on and one of:
- `iterateCount` 0, 32,768 or 4,294,967,295;
- `iterateDelta` -0.1, NaN or a non-number.

Each case's A1 shows the sweeps Excel ran: the oracle calculates three times (recalc.md, "How
excel-graph calculates a probe workbook"), so a count of N shows 3N. Excel may also refuse the
file or repair it; record that too.

## Acceptance criteria
- [ ] Probe cases with oracle values for each setting above.
- [ ] The recalc.md entry cites them. If Excel differs from our fallback, a follow-up task is
      filed (or the entry stays a DEVIATION when the difference is a directive-5 limit).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Bitcode-decoded workbooks (an internal format, not Excel input).
