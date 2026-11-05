"""RustSheet: create, edit and evaluate Excel spreadsheets.

The API is :class:`Workbook`: Excel errors are returned as :class:`XlError` values, edits are
atomic transactions (``with wb.edit() as tx``) and every edit reports a change set in
``wb.changes``. The older ``UserModel`` and ``Model`` classes live in the deprecated
:mod:`rustsheet.legacy` module until the API v1 freeze (SPEC-0026).

Quick start::

    import rustsheet as rs

    wb = rs.Workbook()
    with wb.edit() as tx:
        tx["Sheet1!A1"] = 10
        tx["Sheet1!A2"] = rs.Formula("=A1*2")
    print(wb["Sheet1!A2"])  # 20.0
    print(wb.changes["values"])  # [{"ref": "Sheet1!A2", "before": None, "after": 20.0}]
    wb.save_xlsx("my-workbook.xlsx")
"""

from rustsheet._rustsheet import (
    EditError,
    Formula,
    LoadError,
    RefError,
    RustSheetError,
    SaveError,
    Transaction,
    Workbook,
    WorkbookError,
    XlError,
    __version__,
    column_name_from_number,
    column_number_from_name,
    get_all_timezones,
    get_supported_locales,
    quote_name,
)


def open(path: str, recalc: bool = True) -> Workbook:
    """Open an xlsx file as a :class:`Workbook` (shorthand for ``Workbook.open``)."""
    return Workbook.open(path, recalc)


__all__ = [
    "EditError",
    "Formula",
    "LoadError",
    "RefError",
    "RustSheetError",
    "SaveError",
    "Transaction",
    "Workbook",
    "WorkbookError",
    "XlError",
    "open",
    "__version__",
    "column_name_from_number",
    "column_number_from_name",
    "get_all_timezones",
    "get_supported_locales",
    "quote_name",
]
