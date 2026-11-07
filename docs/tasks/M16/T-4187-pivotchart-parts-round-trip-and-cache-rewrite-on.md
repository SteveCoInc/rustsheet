---
id: T-4187
title: "PivotChart parts: round trip and cache rewrite on export"
milestone: M16
lane: engine
status: done
priority: P3
size: M
tier: opus
depends_on: [T-4186, T-4176]
---

## Goal
Chart parts bound to pivots (c:pivotSource, c:pivotFmts, c14/c15 options) survive import and export, and their caches are rewritten from PivotChartData.

## Context
Depends on chart-part preservation (M11 unknown-part round trip). Program: docs/research/pivot-tables-2026-11-04/plan.md; decision 0274. Coordinate with M15 (xlsx round-trip fidelity, decision 0275): its ownership matrix replays pivot parts verbatim until pivots have an export (docs/research/xlsx-round-trip-2026-11-04.md, `Replay` and `ReplayIfClean` rows); this task is that export, so it hands pivot parts from `Replay` to modelled and uses M15's package validator and roundtrip suite kind where they exist.

## Acceptance criteria
- [x] Round-trip suite case with a PivotChart.
- [x] Records Excel-written chart caches in a committed file (the sidecar or a fixture), so that the claims of
  docs/excel-semantics/pivots.md "Excel for the web rewrites a PivotChart's caches when it renders the pivot", today
  from the oracle's cached download only (probes/t4130-pivot-chart-cache), become repository evidence: the web's
  rewrite of `c:tx`/`c:cat`/`c:val`, and the `#SPILL!` chart's anchor references and one 0 point.
- [x] The value cache's `c:formatCode` as current Excel writes it (pivots.md "A PivotChart's value cache format code
  differs by Excel version").
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Split: this task holds the evidence, the round-trip case and the rewrite; keeping drawings and charts in a save that
regenerates the package, and calling the rewrite there, is T-4950 (size M on its own: M15's `Replay` of a drawing's
closure into the regenerated package).

Evidence: probes/t4187-pivot-chart-caches (excel-graph, 2026-11-07) holds four pivots with a stale one-series column
chart each (pivot_chart.py): C (t4130's), CS (collapsed to `#SPILL!`), F (data field `numFmtId="4"`) and M (Product on
columns). Excel for the web's saved file is committed byte for byte as
eval/roundtrip/excel-saved/cases/t4187-pivot-chart-caches.xlsx (sha256 cc3a1b7a…, the oracle's download for input
sha256 dfa8f50e…), the case of a new gated round-trip suite `roundtrip-excel-saved` (the canary sheet the download
carries keeps it out of the fixtures cell suite, whose oracle fill refuses a second canary). It turns T-4130's claims
into repository evidence and corrects one: `c:tx` reads `'C'!$B$3`, not the category range; `c:cat`/`c:val` read the
item rows and the Grand Total row and cache the items only; the collapsed chart's three references are the anchor
`CS!$A$3` with one 0 point of 2. New: M's one stale series became three (one per column item, `c:tx`
`M!$B$3:$B$4` spanning the header rows, no point where no record reaches), and current Excel writes the data field's
format code (`#,##0.00`), `General` without one (pivots.md, both entries).

Rewrite: `rustsheet_xlsx::export::rewrite_pivot_chart(chart_xml, &PivotChartFrame)` replaces a pivot-bound chart part's
series with one per `PivotChartData` series, references from the layout's location and geometry (sheet quoted by
`quote_name`), caches from the data, `c:formatCode` from the data field; everything else in the part is kept, quotes in
text unescaped (`escape_xml_text`, so Excel's `'C'!$B$3` comes back byte for byte). Its tests reproduce all four of
Excel's saved chart parts byte for byte, both from Excel's own parts and from a part with Excel's series formatting
and the input's stale references and caches; it refuses a part
with no `c:pivotSource`, no series, series in two chart groups, or one nested past 256 levels (the patcher's
`MAX_DEPTH`), and a rewrite past 64 MiB (`MAX_CHART_PART_BYTES`, checked before each piece is appended, so a large
template series times many column lines cannot grow without bound; pivots.md "A rewritten PivotChart part is capped",
a DEVIATION). UNVERIFIED pieces are listed in pivots.md, each with its probe task: T-4938 (several row fields, several
labels, no row or data field) and T-4953, filed here (Values on rows, errors, non-integer spelling, a collapsed pivot
with several series, other chart types, custom, date and percent format codes, quotes in labels).

Eval: probes gains the new case (52 cells, 38 texts, all passing). Round trip: roundtrip-excel-saved records its first
baseline (R0 49.89% kept: the four `refreshOnLoad` pivots render on the first recalculation, so the save regenerates
and drops the drawings, charts and the source's unmodelled parts; R1 and R0/again 100%), which T-4950 lowers;
roundtrip-fixtures unchanged.
