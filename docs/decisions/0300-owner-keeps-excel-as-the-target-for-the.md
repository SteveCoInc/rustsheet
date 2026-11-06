# 0300 — Owner keeps Excel as the target for the open R3 candidates

- **Status:** accepted (2026-11-06, owner ruling). The owner gave it in an interactive Claude Code session
  (https://claude.ai/code/session_01AeCw9bPvaKQB7VyiBFywWu): "I do want to keep matching Excel more than matching the
  math. So for the ones you flagged, the judgment is to match Excel."
- **Context:**
  - Decision 0188 R3 lets the engine follow the math where Excel is more than G = 1e-4 relative off it, but only through
    a decision record carrying an owner ruling. Nine tasks were blocked on that ruling. In each, the engine matches
    Excel today and Gate A holds through Excel's 15 digits:
    - T-4196: BESSELI at a large x and order >= 2, and BESSELJ near a root (engineering.md, T-4083 entry);
    - T-3794: two-argument ERF where the two values nearly cancel (statistics.md, T-1778 entry);
    - T-4277: GAUSS between 1E-15 and 1E-8, where Excel's Phi(z) - 0.5 cancels;
    - T-4317: FISHER and FISHERINV near 0, where Excel's LN and EXP quotients cancel;
    - T-4318: WEIBULL.DIST's density at x = 0, which Excel gives as 0 for every shape;
    - T-4392: GAMMA.DIST's tiny-ratio density, which Excel's log form underflows to 0;
    - T-3484: XIRR near a zero root (financial.md, "XIRR is a bracketing bisection");
    - T-3775: SIN, COS, IMSIN and IMCOS at the 15-digit neighbour of k*pi/2;
    - T-4285: NPER at rates 1E-14 to 1E-13, outside the region decision 0188 R3 already ruled.
- **Decision:**
  - None of the nine is an R3 case. Excel stays the target (directive 1): no `MATH-OVER-EXCEL` entry, no R3 mark, no
    engine change. The engine keeps matching Excel at these points.
  - New points of the same kind that Gate A table tasks find are handled the same way: they are listed in the task's
    Outcome as "Excel kept as the target (decision 0300)", with no owner-ruling task filed for them. A point that would
    need the engine to leave Excel still needs its own owner ruling.
  - Existing R3 cases and rulings are unchanged: decision 0188 R3's initial cases, decision 0213 and decision 0267 item 3.
    T-3544 (T.INV and F.INV points outside the named region) is not covered here, because decision 0267 item 3 already
    lets the math win for T.INV at a tiny p; it stays open for the owner.
- **Consequences:**
  - The nine tasks are done with no code change; the semantics entries that said "until the owner rules" now cite this
    record.
  - Gate A table tasks stop filing owner-ruling tasks for Excel-vs-math gaps.
