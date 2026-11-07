---
id: T-4719
title: "TEXTBEFORE/TEXTSPLIT delimiter lists with two errors in a 2-D list, a final sigma with nothing before it, and a negative REGEX occurrence over a split surrogate pair: probe the text.md UNVERIFIED notes and match"
milestone: M7
lane: functions-text
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel cells settle four notes docs/excel-semantics/text.md leaves UNVERIFIED for the M7 text functions. The engine
matches every probe cell, and text.md states the observed results (or the existing VERIFIED rule that explains them)
in place of the UNVERIFIED wording.

## Context
- Gap triage 2026-11-06 (opus worker run, `make eval` on origin/main 4def66f7 with calipers, corpus-bench and poi
  fetched). `scripts/exit_owners.py --all-off` reports 0 visibly-off cells with no open owner, so the lane's
  UNVERIFIED notes with no open task are the gap left. TEXTBEFORE, TEXTAFTER, REGEXREPLACE and REGEXEXTRACT are M7 in
  eval/catalog/functions.yaml (prior 0.3), TEXTSPLIT M7 (0.5).
- The four notes:
  1. Delimiter lists, row-major or column-major (text.md, the TEXTSPLIT lift entry, "Of two different errors in one
     delimiter list the first element wins (whether the scan is row-major or column-major is UNVERIFIED: every 2x2
     cell has its winner at (1,1)", probes/textsplit-lift-guesses A74-A81; and the TEXTBEFORE delimiter-list entry,
     "The error of a list with two different errors is the first in row-major order (UNVERIFIED)"). Ours:
     `delimiter_list` in crates/rustsheet-engine/src/functions/text/lifted.rs flattens the grid row-major, so
     `{1,#N/A;#DIV/0!,2}` gives `#N/A` and `{1,#DIV/0!;#N/A,2}` `#DIV/0!`; a column-major Excel gives the other.
     The same holds for a multi-cell delimiter range (`evaluate_range`).
  2. Final sigma in TEXTBEFORE/TEXTAFTER's case-insensitive match (text.md, the TEXTBEFORE case-fold entry,
     "UNVERIFIED: whether a letter must precede the `Σ` (the code ignores what precedes it)"). Ours: `sigma_is_final`
     (functions/text/string_format.rs) looks only at the next character, as probes/t3820-lower-sigma-next-char pinned
     for LOWER. Unprobed: a Σ at the start of the text, after a space, after a digit, and after a letter, each before
     a final position (`LOWER("Σ")`, `LOWER(" Σ")`, `LOWER("1Σ")`, `LOWER("aΣ")`), and the same texts through
     `TEXTBEFORE(<text>&"x","ς",,1)` and `TEXTSPLIT` with match_mode 1.
  3. REGEX negative occurrence over a split surrogate pair (text.md, the regex iteration entry, "A negative
     occurrence needs the whole iteration, so it is `#VALUE!` in ours whenever the iteration splits a pair:
     UNVERIFIED (not probed)"). The positive-occurrence cells are probes/regex-iteration-surrogates A1-A11.
  4. TEXTAFTER's array if_not_found through `OFFSET(Data!B9:C12,5,1,DROP(Data!A3:A5,-2,0),2)` (text.md, the TEXTAFTER
     instance_num entry, "UNVERIFIED mechanism (Excel reads the one-cell DROP result as a height giving 2 rows, ours
     as a lifted 1x1)"). Data!A3 is 1E-15, and references.md "OFFSET negative and fractional height/width"
     (VERIFIED, T-0956) already says a height in (0, 1) reaches one cell back, so the OFFSET is 2 rows. The engine
     passes all 70 cells of probes/t2414-gen-m7-unexplained (`make eval-show
     CASE=probes/t2414-gen-m7-unexplained.case.json`), A141:A171 included. This item is a documentation fix: no
     probe is needed unless the rewording claims something the cells do not show.

## Acceptance criteria
- [x] A probe case `probes/t4719-text-unverified-residue.case.json` is committed with excel-graph oracle values and
      holds at least: for item 1, TEXTSPLIT and TEXTBEFORE with the 2x2 delimiter arrays `{"x",#N/A;#DIV/0!,"y"}` and
      `{"x",#DIV/0!;#N/A,"y"}` and with a 2x2 delimiter range holding the same errors, plus a 3x1 and a 1x3 control;
      for item 2, the eight sigma texts above through LOWER and through TEXTBEFORE's case-insensitive match; for
      item 3, REGEXREPLACE with occurrence -1 and -2 and REGEXEXTRACT's all-matches mode over `"a"&UNICHAR(128512)&"b"`
      with the empty pattern and with `x*`, and the same with a pattern that never splits the pair (`b`) as control.
      Surrogate texts are built with UNICHAR, as the existing probe does.
- [x] Every probe cell passes, or each cell that does not is listed in a new functions-text task (`campaign:
      semantics` with a `## Visible-parity stage` section) filed with `make new-task` that names the case and its
      cells, and those known failures are recorded with `make eval-baseline`.
- [x] text.md: items 1-3 state the observed result and cite the probe cells; item 4 cites references.md's fractional
      height rule and t2414 A141:A171 in place of "UNVERIFIED mechanism".
- [x] Any engine change has a unit test that pins a probed result.
- [x] `make check-fast` green; no new known-failure line in eval/baseline/ other than those owned as above; reviewers
      approve.

## Out of scope
- Locales other than en-US and the other case-fold exceptions (İ, Kelvin, Angstrom, ẞ: VERIFIED).
- REGEX pattern features (T-1972, T-2356) and the regex nesting limit.

## Outcome
Probe `probes/t4719-text-unverified-residue` (excel-graph 2026-11-07, 44 cells) passes on the engine with no code change. Results: the delimiter-list error scan is row-major (array and range); a `Σ` is final in `LOWER` whatever precedes it (nothing, space, digit, letter); a negative REGEX occurrence and all-matches over a split surrogate pair are `#VALUE!` for the empty pattern and `x*`, and fine for `b`. text.md states each as VERIFIED with the cells; item 4 cites references.md's fractional-height rule and t2414 A141:A171. Eval delta: one new case, 44/44; no known failures added.
