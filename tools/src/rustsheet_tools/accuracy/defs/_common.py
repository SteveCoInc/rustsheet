"""Helpers shared by the Gate A definitions of several families (T-3748).

Not a function family: the leading underscore keeps `defs/__init__.py` from importing it as one.
"""

from __future__ import annotations

from collections.abc import Sequence

import mpmath

from rustsheet_tools.accuracy.model import MathArg


def scalars(args: Sequence[MathArg]) -> list[mpmath.mpf]:
    """The numeric arguments of a definition that takes no logical or array (its sampler draws numbers only)."""
    out: list[mpmath.mpf] = []
    for a in args:
        assert not isinstance(a, (bool, tuple))
        out.append(a)
    return out


def zero_pole(args: Sequence[MathArg]) -> str | None:
    """One-argument functions with a pole at 0 (COT, CSC, COTH, CSCH): Excel's `#DIV/0!`."""
    (x,) = scalars(args)
    return "a pole at 0" if x == 0 else None
