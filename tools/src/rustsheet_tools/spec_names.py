"""Defined names of a case spec, written with XlsxWriter even when its own name check refuses them (T-4446)."""

from __future__ import annotations

import re

import xlsxwriter

# The names XlsxWriter's `define_name` accepts by their characters (its own check, Python `\w`).
_XLSXWRITER_NAME = re.compile(r"(?!\d)[\w\\][\w\\.]*")


def define_name(wb: xlsxwriter.Workbook, name: str, formula: str) -> None:
    """Defines `name` (`Sheet!name` for a sheet-scoped one) as `wb.define_name` does, also when its
    characters fail XlsxWriter's Python `\\w` check (`ⓐ`, U+1F130, `नाम`, a first `Ⓐ` or mark):
    XlsxWriter drops such a name with only a warning, and the probes of which ones Excel keeps need
    it in workbook.xml (T-4446). A name XlsxWriter refuses as cell-like (`NA1`, `R0C1`) is still
    dropped (T-4752 tracks that)."""
    sheet, _, local = name.rpartition("!")
    if _XLSXWRITER_NAME.fullmatch(local) or not local:
        wb.define_name(name, formula)
        return
    index = -1
    if sheet:
        worksheet = wb.get_worksheet_by_name(sheet)
        if worksheet is None:
            msg = f"defined name {name!r}: no sheet {sheet!r}"
            raise ValueError(msg)
        index = worksheet.index
    # The entry `define_name` appends: [name, sheet index or -1, formula without `=`, hidden].
    wb.defined_names.append([local, index, formula.removeprefix("="), False])
