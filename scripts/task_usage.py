"""The usage tier of a task, for `scripts/tasks.py` (SPEC-0037 design 8, decision 0355 item 7).

A task's tier is the `usage:` of the function its cells belong to in eval/catalog/scope.json:
`foundational` (the functions most workbooks use) or `expert`. Among ready tasks of the same
milestone and priority, `next` picks a foundational one first. Stdlib only, like tasks.py.
"""

from __future__ import annotations

import functools
import json
import re
from pathlib import Path

USAGES = ("foundational", "expert")  # the `usage:` values
SCOPE = Path("eval") / "catalog" / "scope.json"
# A run of name characters that is not the tail of a longer name, a hyphenated word, a sheet or
# absolute reference (`P!SUM`, `$SUM`) or an error literal (`#N/A`), and the character after it.
# The name is a whole word when that character is `(`, `,`, a space or the end (SPEC-0037).
NAME = re.compile(r"(?<![\w.#!$-])([A-Z][\w.]*)(?=([(, ]|$))")
# Names that a title also uses for something else: a letter (`N and T`, a population size `N 6`,
# `Top-N`), a value (`a TRUE cell`, `a FALSE sort_order`) or "not available". They count only as
# a call, before `(`. Other short names count as words: the match is case-sensitive, and in the
# task titles of 2026-11 an all-caps IF, AND, OR, NOT or DAY is always the function.
CALL_ONLY = frozenset({"N", "T", "NA", "TRUE", "FALSE"})


@functools.cache
def catalog_tiers(root: Path) -> dict[str, str]:
    """Function name -> `usage` from the tree at `root`, read once per root; empty without the
    catalog (a test tree, or a checkout before T-5091)."""
    path = root / SCOPE
    if not path.is_file():
        return {}
    rows = json.loads(path.read_text()).get("functions", [])
    return {str(r.get("name")): str(r["usage"]) for r in rows if isinstance(r, dict) and r.get("usage") in USAGES}


def title_usage(title: str, root: Path) -> str | None:
    """The tier of the first catalog name that is a whole word of `title`, left to right; None
    when there is none. A title names its subject function first: a later name is usually one its
    formula quotes (T-3061's `T.INV is SQRT(...)` is a T.INV task)."""
    tiers = catalog_tiers(root)
    found = (tiers.get(name) for name, after in NAME.findall(title) if name not in CALL_ONLY or after == "(")
    return next((usage for usage in found if usage is not None), None)


def effective_usage(t: dict[str, object], root: Path) -> str | None:
    """Task `t`'s tier: its `usage:` key, else the tier its title names, else None."""
    usage = t.get("usage")
    return str(usage) if usage in USAGES else title_usage(str(t.get("title", "")), root)


def usage_note(t: dict[str, object], root: Path) -> str:
    """What `next` and `ready` add to a foundational task's line."""
    return "  usage: foundational" if effective_usage(t, root) == "foundational" else ""
