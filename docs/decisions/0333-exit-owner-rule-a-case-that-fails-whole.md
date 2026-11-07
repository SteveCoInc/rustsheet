# 0333 — Exit-owner rule: a case that fails whole is owned by any block that names it

- **Status:** accepted (2026-11-07, T-3438). Amends decision 0159 (and 0250's restatement of the whole-case rule).
- **Context:**
  - A case can fail whole: its baseline line is `!EngineError` or `!Panic`, or it did not run. `exit_owners.py`
    lists it as `case!*` (`WHOLE_CASE`), with no A1 name.
  - Decision 0159 made a single A1 name own a cell only in a block that names its case, and kept 0096's rule that a
    line naming the case with no A1-shaped token owns the whole case. Neither covered a whole-case failure next to a
    cell list. So in T-3338's bullet, which wraps over five lines naming 12 `t3266-nested-*` cases that fail whole
    and ends with `: S!A1, S!B1`, the 10 cases on the first lines were owned and the 2 on the last line were not. A
    cell list, the form decision 0159 asks planners to write, could never own a crash on its own line.
  - Measured on `make eval` at 42d9bdaa (with T-4739's OFFSET change, which moves no owner) (T-3438): also reading the 0096 whole-case rule per block (T-3438's proposed
    rule 1) would leave 17 visibly-off cells newly unowned (probes-numeric/fit-audit-t3271-cdf-cut S!A10, A14:A16,
    A35:A39 and t3316-gamma-cdf-huge-shape S!A2:A6, A17:A19), whose owners name the case on a line of its own and
    list cells on a continuation line. The rule below alone changes no cell's owner in that report.
- **Decision:**
  - A `WHOLE_CASE` failure is owned by any block of an open task that names its case, with or without A1 names
    beside it. The failure hides every cell of the case, so the task that owns those cells owns the failure too.
  - Everything else is unchanged. A cell-level failure needs its A1 name, or a colon span holding it, in a block that
    names its case (0159, 0250). A **line** (not a block, despite 0250's wording) that names the case with no
    A1-shaped token owns every cell of the case (0096), as `names_whole_case` has always read it.
- **Consequences:**
  - `owned` in `scripts/exit_owners.py` returns true for a `WHOLE_CASE` cell once the text names its case. It is the
    one rule `make exit-owners`, `make unowned-off` and `scripts/desktop_owned.py` share, so all three change
    together. On the 42d9bdaa report none of their outputs changes.
  - A task no longer needs a separate bullet naming a crashing case whole (T-3338's workaround).
  - A task that mentions a crashing case in passing owns its crash. That was already so for a line naming the case
    with no A1 name; a reviewer of a task's cell list still checks the owner is the right one.
  - Tests: `scripts/test_exit_owners.py` (the T-3338 shape; a case named in no block stays unowned; 0159's leak test
    with a crash beside it).
