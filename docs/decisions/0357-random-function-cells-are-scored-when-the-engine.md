# 0357 — Random-function cells are scored when the engine shows their value does not depend on the draws

- **Status:** accepted (2026-11-08, T-5100). Asked for by the repository owner in chat
  (2026-11-08): stop skipping volatile functions wholesale in eval. Amends T-0005's skip rule for
  RAND, RANDBETWEEN and RANDARRAY. Keeps decision 0225 (these three functions pass Gate B on their
  draw-independent cells) and widens which cells count as draw-independent.
- **Context:**
  - The harness skipped every cell whose formula, or a precedent's, calls a random function,
    unless a case listed it in `invariant` (T-1283). The skip went by the formula text, so a cell
    whose answer cannot depend on the draw went unscored: `ISNUMBER(RAND())`,
    `IF(RAND()>2,1,NA())`, a wrapper or LAMBDA that hands a RAND result to a function that ignores
    it. A wrong answer there stayed hidden. One review found such a case (0 where Excel gives
    `#N/A`), and the only fix on offer was to skip the wrapper too.
  - Excel's draws cannot be reproduced. Microsoft documents neither the seed (nor a way to set
    it), nor how a generator output becomes a double, nor how RANDBETWEEN and RANDARRAY derive
    their values from draws. With multithreaded recalc the order in which cells use up draws is not
    fixed, and every recalc draws again (the oracle's double fill sees the values change, T-1283).
    A bit-exact copy of Excel's generator would still match no cell.
  - The engine drew from the host's unseeded generator, so two harness runs never agreed and the
    harness could not tell which values depend on the draws.
- **Decision:**
  1. The engine draws RAND, RANDBETWEEN and RANDARRAY from a per-model `RandomSource`: the host's
     generator by default (unchanged for every user), a seeded one, or a fixed draw.
  2. The harness evaluates every case with a fixed seed. A cell that reads a random draw, directly
     or through a precedent, and no clock (NOW/TODAY unpinned) is scored only if its engine value is
     the same under each of four alternate sources: two other seeds, a fixed draw of 0 and a fixed
     draw of the largest double below 1. The sources are fixed, so every run scores the same cells.
     Any other such cell is skipped as volatile, as before. The `invariant` list and the clock rule
     are unchanged.
  3. A draw-dependent value is never compared with Excel. The rule does not ask the engine to
     reproduce Excel's draws, and this is no deviation: Excel's behavior is "a new draw on every
     recalc", which no single value can show.
- **Consequences:**
  - Cells that reach a random function through a precedent but do not depend on its draws are
    scored now. Each one that fails is new coverage (its case's checked count changes) and gets a
    task, like any other.
  - A cell that is only sometimes steady (`RANDBETWEEN(1,2)=1`) could agree across all five runs by
    chance. The two fixed ends make that unlikely for a value that compares a draw with a threshold.
    The result is still deterministic, so a miss shows as a stable failure, never a flaky one.
  - The engine being steady does not prove Excel's cell is. When the engine is wrong in a way that
    makes a cell steady while Excel's value moves with the draw, the cell is scored and fails. The
    failure is real (the engine's value should depend on the draw), but Excel's value is one draw.
    The oracle fill does not yet fill such cells twice (it does so only for `invariant` cells), so
    a failure of that kind is triaged by hand.
  - A case with random-reading cells is evaluated five times, inside the same timeout.
  - The same check is open for NOW/TODAY, by varying the clock; that is not done here.
