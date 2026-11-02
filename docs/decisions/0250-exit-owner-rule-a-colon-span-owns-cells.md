# 0250 — Exit-owner rule: a colon span owns cells in the block that names its case

- **Status:** accepted (2026-11-02, T-3623). Amends decision 0159 (and through it 0136's colon-span rule).
- **Context:**
  - Decision 0159 made a single A1 name own a cell only in a **block** (a line plus its continuation lines) that
    names the cell's case, but kept 0136's rule that a colon span counts only on the case's own physical line.
  - Tasks wrap long cell lists onto indented continuation lines of the same bullet (T-3479 lists t3354-f-dist cells
    as `A153:A166`-style spans on the wrapped lines). Those spans were reported as unowned by `make exit-owners`
    even though the single names beside them were owned, which invites triagers to file duplicate tasks.
  - The per-line rule existed so that a quoted formula range (`Data!B6:C10`) does not make a task own those cells.
    A quoted formula sits in another block (another bullet or paragraph), so a per-block rule keeps that property.
- **Decision:**
  - A colon span owns the cells of its rectangle only in a block (as decision 0159 defines it) that also names the
    cell's case, exactly like a single A1 name.
  - The rest of the rule is unchanged: a block naming the case with no A1-shaped token owns the whole case (0096);
    hyphen spans name only their two ends (0136).
- **Consequences:**
  - `names_in_span` in `scripts/exit_owners.py` iterates `blocks()`, the same block definition `names_beside_case`
    uses; there is still one definition of a block. `make exit-owners`, `make unowned-off` and
    `scripts/desktop_owned.py` all go through `owned`.
  - A span on its own bullet, or in a paragraph that does not name the case, still owns nothing.
  - Tests: `scripts/test_exit_owners.py` (a span on a continuation line is owned; a span in another block is not).
