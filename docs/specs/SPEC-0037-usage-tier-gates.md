# SPEC-0037 — Usage-tier gates and the envelope ratchet

- **Status:** accepted (2026-11-08, decision 0355)
- **Milestone / lane:** M0 (Phase 1 exit machinery) / harness, quality
- **Tasks:**
  - T-5091 (this spec);
  - the implementation tasks listed under Rollout.

## Problem

Decision 0355 replaces Phase 1's Gate A and Gate B with gates by **usage tier**: foundational and expert functions.
It also replaces the bit-level CI ratchet with an **envelope** ratchet. Today:

- **The envelope is a 15-digit rule** (`accuracy/envelope.rs`), and it needs a math value. Gate A, R3 and R1(c)
  share it. The new envelope is 12 digits of Excel, or 1e-9 of the math. Eval cells without a math value must
  still be judged on the Excel side.
- **Nothing in the report knows a function's tier.** The catalog now carries `usage:` (decision 0355 item 1), but
  no report section, `scripts/phase1_exit.py` line or dashboard block reads it.
- **CI blocks every new bit-level failure.** `baseline::diff` calls a new `F` line in an unchanged case a
  regression. The visible-parity lock caps each case's visibly-off cells at 15 digits. A change that moves a cell
  inside the envelope is blocked, while a change that pushes a known failure further off passes unseen.
- **Task picking ignores the tier.** `scripts/tasks.py next` orders tasks by P0, handoff, re-review, campaign,
  milestone, priority and id.

The measured state at main b84a7256 is in decision 0355's Context.

## Goals / non-goals

**Goals:**
- One envelope definition, used by:
  - the accuracy tables;
  - R1(c) and the milestone rows;
  - the tier gates;
  - the CI ratchet.
- Tier gate rows in report.json and report.md, the Phase 1 exit lines, the dashboard, and the steward's daily
  status.
- A CI gate that allows bit drift inside the envelope and blocks:
  - any cell leaving it;
  - any type or error going wrong;
  - any tier gate flipping from met to unmet.
- Task ordering that prefers foundational work when priority is otherwise equal.

**Non-goals:**
- Moving functions between tiers (a decision record does that).
- Fixing cells.
- Changing the oracle or the fit guards (decision 0188 R4).

## Excel behavior to match

None: this is measurement machinery. Prime directive 1 is unchanged: the engine's target is still Excel. The
envelope is a gate tolerance, not a behaviour target.

## Design

### 1. The tier (done in T-5091's PR)

- eval/catalog/functions.yaml writes `usage: foundational` on the 122 functions of decision 0355's rule. Absent
  means expert.
- `rustsheet-catalog-view` writes `usage` on every scope.json row, with `expert` as the default.
- `catalog::Usage { Foundational, Expert }` reads it. tools/tests/test_catalog.py holds the catalog to
  eval/catalog/usage.tsv.

### 2. The envelope: one definition in `accuracy/envelope.rs`

- **New constant:** `pub const ENVELOPE_DIGITS: i32 = 12;`. The Excel side is
  `closeness::within_significant_digits(excel, engine, ENVELOPE_DIGITS)`, which replaces
  `Visible::is_off_at_fifteen_digits` in `envelope()`. Rename `Envelope::Excel15` to `Envelope::Excel` (serialized
  `excel`), and update every reader of accuracy.json (`tools/`, `scripts/`) in the same change.
- **New per-cell classifier** for eval cells, built from the same pieces as `envelope()`:

  ```rust
  pub enum CellEnvelope { Inside, Outside, WrongType }
  pub fn cell_envelope(c: &CellResult) -> CellEnvelope
  ```

  - **Inside:**
    - a bit-exact pass, or a pinned deviation;
    - two numbers within `ENVELOPE_DIGITS` of each other;
    - a number within ε (or the absolute floor) of `c.math`;
    - an R3 cell (`Target::Math`) within ε of the math.
  - **WrongType:**
    - a number against an error, either way;
    - two different error codes;
    - a type mismatch, text that differs, or a different boolean.

    The rule is the closeness buckets' `visible_other`, at any number of digits.
  - **Outside:** two numbers that are inside on neither side.
- `cell_envelope` is the only per-cell envelope test. The tier gates (3), R1(c) (4), the ratchet (6) and the
  visible lock (6) call it. Never write a second one (CLAUDE.md directive 3).
- **R1(c)** (`r1c::classify`): a numeric cell counts only if it is `Outside`; a `WrongType` cell always counts.
  The "Excel more than ε off the math" clause goes: decision 0355 item 2 makes a value within ε of the math good,
  whatever Excel says. `R1c::Phase1b` keeps its meaning: inside the envelope, but not bit-exact.

### 3. The tier gates: report.json `usage_gates`, report.md "Usage tiers"

- **New module** `usage_gates.rs`, built in `RunReport::build` beside `v15_checkpoint`.
- **Cells counted:**
  - the suites `probes`, `probes-numeric`, and `v15_checkpoint`'s Gate B suites;
  - attributed as decision 0118 says (`attribution`);
  - leaving out decision 0134 exceptions (`phase1_exceptions`), deviations, pivot suites (`gen-pivot`,
    `pivot-corpus-*`, `pivot-export`) and GETPIVOTDATA (decision 0354).
- **Accuracy-table points** are added by `scripts/phase1_exit.py` from eval/runs/accuracy.json, each point under
  its table's function. They are not added in the report, which does not run the accuracy check.

The JSON schema:

```json
"usage_gates": {
  "suites": ["probes", "probes-numeric", "gen-m1", "..."],
  "tiers": [
    {"usage": "foundational", "functions": 122, "cells": 2119054, "wrong_type": 31, "outside": 1,
     "type_rate": 0.9999854, "envelope_rate": 0.9999995,
     "pooled_bar": {"type": 1.0, "envelope": 0.9999},
     "function_bar": {"type": 1.0, "envelope": 0.0},
     "failing_functions": ["TEXT", "EDATE", "NPV"], "met": false},
    {"usage": "expert", "functions": 384, "cells": 902063, "wrong_type": 1355, "outside": 405,
     "type_rate": 0.998498, "envelope_rate": 0.999551,
     "pooled_bar": {"type": 0.999, "envelope": 0.999},
     "function_bar": {"type": 0.99, "envelope": 0.99},
     "failing_functions": ["IRR", "RATE", "..."], "met": false}
  ],
  "functions": [
    {"name": "IRR", "usage": "expert", "cells": 3768, "wrong_type": 930, "outside": 9,
     "type_rate": 0.7532, "envelope_rate": 0.9976, "meets": false}
  ],
  "met": false
}
```

- **The bars** (decision 0355 item 3):
  - foundational: 100% type pooled and ≥ 99.99% envelope pooled, with no per-function envelope bar;
  - expert: ≥ 99.9% pooled on both measures, and ≥ 99% per function on both.
- In `function_bar`, `0.0` means no per-function bar: the foundational envelope bar is pooled only. Its
  per-function type bar of 1.0 follows from the pooled 100%.
- A function with no counted cell is listed with `cells: 0` and `meets: true`. Coverage and the evidence floor
  gate it elsewhere.
- Rates are `(cells − misses) / cells`.
- `met` holds when every tier is met.
- **report.md** gets a "Usage tiers" section. It has one table row per tier (functions, cells, type rate, envelope
  rate, bars, met), then each failing function with its counts, worst first.

### 4. Milestone rows (decision 0355 item 4)

- `probe_milestones` and `milestone_exit` count the cells under the new R1(c) (2).
- A milestone's probe row and its ironcalc-calc row are met when the cells of the milestone's functions meet the
  tier bars of (3), pooled within the row. A row whose counted cells are all expert and within the bars is met.
  Text items (M10) and spill-extent failures (M7) keep their rules: they are type and text, so they must be zero
  for foundational functions.
- Bracketed bit-level counts stay, as Phase 1b information.

### 5. `scripts/phase1_exit.py`

- The lines `gate A` and `function rate` are replaced by:
  - `foundational`: met or not, with the rates and the failing functions;
  - `expert`: likewise.

  Both read `usage_gates` plus accuracy.json's points, per tier.
- The old Gate A (accuracy holds at 15 digits) and Gate B (99.9% at 15 digits) are printed after them, marked
  `info`, and gate nothing.
- Unchanged:
  - `owners`;
  - coverage;
  - the milestone lines (now under (4));
  - the JSON shape, with keys added only.

### 6. The CI gate: an envelope ratchet

- **Baseline lines.** A new kind, `E<TAB>case<TAB>Sheet!A1`, lists a known failure whose cell is `Outside` or
  `WrongType` under `cell_envelope`. Every `E` item is also an `F` item.
  - `F` keeps listing every bit-level known failure, so Phase 1b keeps its to-do list.
  - Whole-case items (`*`, `!status`) are always `E`: a crashed or missing case is never drift.
  - A case in a `cases`-mode suite (corpus-bench) fails `*` only when one of its cells is `Outside` or
    `WrongType`. A case with only drift is not a failure there.
  - Text (`TF`) and layout (`LF`) lines are unchanged: they are type and text, so exact.
  - A third kind, `W<TAB>case<TAB>Sheet!A1`, lists the `E` items that are `WrongType` (and every whole-case
    item), so an `E` item outside as a number that becomes a wrong type, error or text is a regression too;
    a `W` item that stays outside as a number is **right type**, recorded like a fix (T-5093).
- **`baseline::diff`.** In an unchanged case:
  - a **regression** is an item that is new in `E`;
  - **drift** is an item new in `F` but not in `E`. Drift is allowed, recorded by `update-baseline` without
    `--allow-regressions`, and printed as `drift case Sheet!A1`;
  - an item that leaves `E` but stays in `F` is **into the envelope**, recorded like a fix;
  - the existing kinds (fixed, new coverage, new cases, removed cases) are unchanged.
- **The check.**
  - `rustsheet-eval check` fails on any regression, as today.
  - It still fails while the baseline is out of date, so the PR's diff shows every drift line for review.
  - Each suite's summary gains `drift N` and `into envelope N`.
- **Visible-parity lock** (`visible_parity.rs`). It counts each case's `Outside` and `WrongType` cells instead of
  its visibly-off cells. The lock file's header text says so. The first `update-baseline` after the change lowers
  it to the new counts; raising it still needs `--allow-regressions`.
- **Accuracy check.** It uses the 12-digit envelope (2). In the same PR, the known-failure lines that become
  stale are removed with the check's own output, never by judgment (at b84a7256 about 49 of 118).
- **Tier gate lock.** eval/baseline/usage-gates.txt holds one line per tier: `foundational<TAB>met|unmet`, and the
  same for expert.
  - `check` on a full gated run fails when a tier written `met` is unmet.
  - `update-baseline` writes `met` when a tier becomes met, and writes `unmet` over `met` only with
    `--allow-regressions`.
  - The gated run only covers the vendored suites. It computes the tiers on those cells and labels them as
    such; the full report's tiers in (3) are the exit measure.
- **Text.**
  - CLAUDE.md prime directive 2 reads: "`make check` must pass: no cell may leave the decision 0355 envelope or
    get a wrong type or error, and no tier gate may go from met to unmet; bit drift inside the envelope is
    recorded, not blocked."
  - docs/process/quality-gates.md describes the drift, regression and into-envelope kinds.
  - The baseline header text names the `E` kind.
- **Migration.** One `make eval-baseline` run writes the `E` lines into every baseline. That is a tool-generated
  change to protected files under decision 0355, with no hand edits.

### 7. Reports

- **Progress dashboard** (`tools/src/rustsheet_tools/progress_report/`, SPEC-0022). A "Usage tiers" block goes
  right after the bottom line:
  - one row per tier: the rates against the bars, and met or not;
  - the failing functions, worst first, top 10 per tier;
  - the change since the previous dashboard, from eval/runs/latest/phase1-exit.json.

  The Phase 1 checklist block lists the tier lines in place of Gate A and Gate B. A test pins the block from a
  fixture report.
- **Skills.** The `progress-report` skill's bottom line and real-world-readiness judgments cite the tier gates.
  The `steward` skill's daily status roll-up (docs/status.md) leads with the tier gate line, for example
  "foundational 31 wrong type, 1 outside: not met; expert …".

### 8. Tasks

- **Front matter.** A new optional key, `usage: foundational | expert`.
  - `scripts/tasks.py` validates its value.
  - docs/process/task-format.md documents it: the tier of the function the task's cells belong to. A task that
    touches no particular function has no `usage`.
- **Effective tier**, `effective_usage(task)`, in order:
  1. the `usage:` key;
  2. else, from the title: the tier of the first catalog function name in the title that appears in it as a
     whole word (an upper-case name followed by `(`, `,`, space, or the end);
  3. else none.

  The names come from eval/catalog/scope.json, read once.
- **Ordering.** `available()`'s sort key gains `effective_usage(t) != "foundational"` right after
  `str(t.get("priority"))`. Foundational tasks therefore come first among tasks of the same milestone and
  priority, and the rest keep their order. P0, handoffs, re-reviews and campaigns still come first.
- **Output.** `next` and `ready` print `usage: foundational` on a foundational task's line.
- **Creating tasks.**
  - `tasks.py new` takes `--usage foundational|expert`.
  - Without the flag it writes the tier inferred from the title, when there is one.
  - The `planner` and `gap-triager` agent definitions say to set `usage:` on function tasks.
  - The `work-loop` skill's step 1 notes the ordering.

## Alternatives considered

- **Pooled gates without the per-function bar.** This lets one broken expert function (IRR at 75% type match) hide
  inside 99.9%. The owner chose the per-function bar (2026-11-08).
- **Inferring task tiers from case cells** (`exit_owners`) instead of titles. That is more precise, but slow:
  it needs a report. The explicit `usage:` key covers the cases where the title misleads.
- **Dropping `F` lines for drift cells.** That loses Phase 1b's to-do list and the commit trace of each drift.

## Test plan

- `accuracy/envelope.rs`: unit tests at the 12-digit edge (half a unit in the 12th digit, both sides), the
  absolute floor, error against number, and R3.
- `cell_envelope`: one test per class.
- `usage_gates`: fixtures for each bar, with a function at exactly 99% and 99.9%.
- `baseline::diff`: drift, regression, into-envelope, whole-case items, and `cases` mode.
- The visible lock counting `cell_envelope`, and the tier lock flipping.
- `scripts/test_phase1_exit.py`: the tier lines.
- `scripts/test_tasks.py`: `effective_usage` (key, title inference, none) and the ordering at equal priority.
- The progress_report fixture test.
- The worked example: the tier gates on the b84a7256 report match decision 0355's table within the expert cut's
  stated counts.

## Rollout

Each task below is S or M, one lane. The old gates stay in force until each piece lands.

| # | Task | Lane | Depends on | What |
|---|---|---|---|---|
| 1 | T-5091 | harness | none | this spec; `usage:` in the catalog, scope.json and `catalog::Usage` (design 1); `ENVELOPE_DIGITS` and `cell_envelope` in accuracy/envelope.rs, not yet called (design 2) |
| 2 | T-5092 | harness | 1 | the envelope at 12 digits and `cell_envelope` (2); R1(c) and the milestone rows (4); accuracy stale lines; `usage_gates` (3); phase1_exit lines (5) |
| 3 | T-5093 | harness | 1, uses 2's `cell_envelope` | the envelope ratchet: `E` lines, drift, the visible lock, the tier lock, CLAUDE.md directive 2, quality-gates.md (6) |
| 4 | T-5094 | quality | 2's schema | dashboard "Usage tiers" block; progress-report and steward skills (7) |
| 5 | T-5095 | quality | 1 | task `usage:`, `effective_usage`, ordering, `new --usage`, agent and skill text (8) |

Tasks 2 to 5 can be built in parallel: `cell_envelope` lands with task 1, and the schema above is fixed. Task 2
switches `envelope()` and R1(c) to 12 digits; task 3 calls `cell_envelope` for the ratchet and the lock.
