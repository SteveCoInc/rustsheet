"""What the two pivot builders share (pivot.py, pivot_definition.py; T-4122): the OOXML namespaces and how
a cached number is written."""

from __future__ import annotations

MAIN_NS = "http://schemas.openxmlformats.org/spreadsheetml/2006/main"
REL_NS = "http://schemas.openxmlformats.org/officeDocument/2006/relationships"
PACKAGE_REL_NS = "http://schemas.openxmlformats.org/package/2006/relationships"


def number_text(value: float) -> str:
    """A cached number's text: a whole number below 1e15 without a fraction, else the shortest repr."""
    return str(int(value)) if float(value).is_integer() and abs(value) < 1e15 else repr(float(value))
