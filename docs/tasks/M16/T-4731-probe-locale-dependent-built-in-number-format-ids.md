---
id: T-4731
title: "Probe locale-dependent built-in number format ids and custom id numbering on save"
milestone: M16
lane: engine
status: done
priority: P3
size: S
tier: opus
depends_on: [T-4169]
---

## Goal
The engine's built-in number format table and its custom-id numbering are backed by Excel evidence beyond id 14.

## Context
T-4169 changed `DEFAULT_NUM_FMTS[14]` to `m/d/yyyy` on evidence (probes/t4169-pivot-number-formats P1!D5:D11) and made the engine number new custom formats from 164 (`get_new_num_fmt_index`, crates/rustsheet-engine/src/number_format.rs). The 164 start is a convention matching XlsxWriter and ECMA-376 §18.8.30 (ids below 164 are built-in or reserved); how Excel itself numbers custom formats when it saves is not probed. Also unprobed: how Excel en-US shows the other built-ins that may depend on the locale (15-17, 22, and the East Asian 27-36 and 50-81). number-formats.md "Built-in id 14 is the short date" and "Custom number format ids start at 164 (convention)" list them.

## Acceptance criteria
- [x] A probe of plain (non-pivot) cells styled with ids 14, 15-17, 22, 27-36 and 50-81 records Excel's displayed text, filled with excel-graph; the engine matches it or each off cell has an owner.
- [x] A workbook saved by Excel shows which ids it gives new custom formats; number-formats.md records it.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Locales other than en-US.

## Outcome
probes/t4731-builtin-num-fmt-ids (excel-graph, 2026-11-07) shows ids 14, 15-17, 22, 27-36 and 50-81 on
43831.75, 1.25 and -1234.5 in plain cells (B!B1:D47, 147 text cells). Excel en-US shows every East Asian id
as an en-US built-in and saves it as that id; 22 is `m/d/yyyy h:mm` (four-digit year) and 47 `mm:ss.0`,
not the ECMA codes. `DEFAULT_NUM_FMTS` now has Excel's 22 and 47, and `get_num_fmt` shows each East Asian
id as its en-US id (`en_us_builtin_id`; test first); the case passes all 147 text cells. Its download is committed as
eval/roundtrip/excel-saved/cases/t4731-builtin-num-fmt-ids.xlsx (decision 0342): Excel kept the file's
custom ids 164 and 165 and gave the two formats typed entries added 166 and 167, as
`get_new_num_fmt_index` does. number-formats.md: "Built-in id 14 is the short date", "Built-in ids Excel
en-US maps to other built-ins", "Custom number format ids on save". spec2xlsx takes an integer
`number_formats` value as a bare built-in `numFmtId`; the eval adapter mirrors it through the engine's
table (`NumberFormatSpec`). Eval: 0 regressions, probes +1 case (passing), roundtrip-excel-saved +1 file
(R0 kept in full; its R1/E5 and R1/all lines, recorded by `make eval-baseline`, are SPEC-0034 D5 delta
violations: the download has no sharedStrings.xml, so the patcher regenerates the package; owner T-4831).
The mapping is display-only: the engine still saves an East Asian id as itself and writes no `numFmt`
entries for 5-8 (owner T-5056). `get_num_fmt` no longer indexes the table with a negative id.
Follow-ups: T-5042 (custom ids with a gap or above 164), T-5043 (ids 23-26 and 82-163, a typed
`m/d/yyyy h:mm` cell, 1904 cell display), T-5056 (save the en-US ids and the 5-8 entries).

## Handoff (parked 2026-11-07)
Rebased onto origin/main 650099595 (no conflicts; `make eval-baseline` left every baseline file unchanged).
Review items (fidelity review of the first version), all done in the amended commit:
- MAJOR 1, done: number-formats.md "Built-in ids Excel en-US maps to other built-ins" has an Ours line (the
  engine keeps the file's id on save, `cell_xf_xml` writes 27 as 27, and writes no `numFmt` entries for
  5-8), Implemented-in says display only, and T-5056 (filed) owns the save side.
- MAJOR 2, done: dates.md says "in the 1900 system", cites only the date cells B!D1:D24 and D37:D47, and
  keeps 1904 cell display UNVERIFIED with owner T-5043 (an acceptance criterion added there).
- MINOR 3, done: the "ENTERED log above" citation is replaced by docs/research/t3891-typed-script-digit-shapes-2026-11-04.md
  lines 12 and 14, typed-currency-percent-input-2026-10-11.md line 8 and typed-clock-shapes-2026-10-19.md line 17.
- MINOR 4, done: the Outcome names T-4831 as owner of the new roundtrip-excel-saved R1/E5 and R1/all lines
  (SPEC-0034 D5 delta violations: no sharedStrings.xml).
- MINOR 5, done: `m/d/yyyy h:mm` -> 22 in `get_default_num_fmt_id` is UNVERIFIED in the entry, and T-5043's
  acceptance criteria gain a typed `1/2/2020 10:00` cell.
- MINOR 6, done: `get_num_fmt` indexes through `usize::try_from(..).ok().and_then(get)`, General otherwise;
  unit test for -1 and `i32::MIN` in `east_asian_builtin_ids_are_their_en_us_builtins`.
Gates on the amended commit: `make eval-baseline` 0 regressions in the cell suites; `cargo test -p
rustsheet_engine number_format` 25 passed; `cargo test -p rustsheet_eval` 467 passed; `make eval-gates`
exit 0 and `make py-check` exit 0 when each was run on its own. Not re-reviewed yet.
`make check-fast` then passed (exit 0) on this commit's code, before this Handoff section was added.
For the next worker: `make check-fast` had failed five times at eval-gates on one roundtrip-poi case,
`LIBRE_OFFICE-116306-0.xlsx#R0`, `!Timeout` (60 s per case), while other agents' eval runs shared the
4-core machine. The same gate passed when run alone (`roundtrip --check`, `make eval-gates`), and that case
takes about 29 s with both this branch's and origin/main's binaries (timed back to back), so the change
does not slow it. If it fails again, rerun on a quiet machine. Next step: run the review skill again.

