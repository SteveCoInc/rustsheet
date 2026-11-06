"""Type stubs for the native extension module ``rustsheet._rustsheet``.

The extension also defines the deprecated legacy classes; their stubs live in
``legacy.pyi`` (SPEC-0026) and are re-exported here.
"""

from typing import Any, Literal

from rustsheet.legacy import CellType as CellType
from rustsheet.legacy import Model as Model
from rustsheet.legacy import UserModel as UserModel
from rustsheet.legacy import create as create
from rustsheet.legacy import create_user_model as create_user_model
from rustsheet.legacy import create_user_model_from_xlsx as create_user_model_from_xlsx
from rustsheet.legacy import load_from_xlsx as load_from_xlsx

__version__: str

class WorkbookError(Exception): ...
class RustSheetError(Exception): ...
class RefError(RustSheetError): ...
class EditError(RustSheetError): ...
class LoadError(RustSheetError): ...
class SaveError(RustSheetError): ...
class PasswordRequiredError(LoadError): ...
class WrongPasswordError(LoadError): ...

class XlError:
    def __init__(self, literal: str) -> None: ...
    @property
    def literal(self) -> str: ...
    @property
    def code(self) -> int | None: ...
    @property
    def detail(self) -> str | None: ...

class Formula:
    def __init__(self, text: str) -> None: ...
    @property
    def text(self) -> str: ...

CellValue = float | str | bool | XlError | None
EditValue = float | str | bool | XlError | Formula | None

class Transaction:
    def __enter__(self) -> Transaction: ...
    def __exit__(self, exc_type: Any, exc: Any, tb: Any) -> bool: ...
    def set(self, ref: str, value: EditValue) -> None: ...
    def __setitem__(self, ref: str, value: EditValue) -> None: ...
    def clear(self, ref: str) -> None: ...
    def set_number_format(self, ref: str, format: str) -> None:
        """Give the cell `ref` the number format `format`, keeping its value and other style.

        A multi-cell target, a target outside the sheet or an unknown sheet raises an error when
        the `with` block exits and rolls the block back.
        """
    def set_row_hidden(self, sheet: str, row: int, hidden: bool = True) -> None:
        """Hide (or show) a row of `sheet`; `row` is the 1-based row number Excel shows.

        A row outside 1..=1048576 raises EditError at once; an unknown sheet raises RefError when
        the `with` block exits.
        """
    def add_table(
        self, range: str, name: str, filtered: bool = False, totals_row: bool = False
    ) -> None:
        """Add an Excel table over `range` (its first row is the header row).

        EditError (invalid or taken name, overlap, no data row, bad header) or RefError (unknown
        sheet) surfaces when the `with` block exits and rolls the block back.
        """
    def add_sheet(self, name: str) -> None: ...
    def rename_sheet(self, name: str, new_name: str) -> None: ...
    def delete_sheet(self, name: str) -> None: ...
    def define_name(self, name: str, formula: str, sheet: str | None = None) -> None:
        """Define a workbook-scoped name, or one scoped to `sheet` (shadows a workbook name there).

        Errors (EditError for a duplicate or invalid name or an unparsable formula, RefError for an
        unknown sheet) surface when the `with` block exits and roll the block back.
        """
    def add_data_table(
        self,
        range: str,
        row_input: str | None = None,
        column_input: str | None = None,
    ) -> None:
        """Add a what-if data table with result cells `range`; the next evaluation fills them.

        Errors (EditError) surface when the `with` block exits and roll the block back.
        """

class Workbook:
    def __init__(
        self,
        date_system: Literal["1900", "1904"] = "1900",
        clock: int | None = None,
        name: str | None = None,
    ) -> None: ...
    @property
    def date_system(self) -> Literal["1900", "1904"]: ...
    @property
    def clock(self) -> int | None:
        """Fixed clock, ms since the Unix epoch (UTC); None is the system clock.

        Setting it is one edit: NOW/TODAY recalculate and `changes` is set. An int outside
        1900-01-01..=9999-12-31 raises ValueError; a bool or non-int raises TypeError.
        """
    @clock.setter
    def clock(self, value: int | None) -> None: ...
    @staticmethod
    def open(
        path: str,
        recalc: bool = True,
        clock: int | None = None,
        name: str | None = None,
        password: str | None = None,
    ) -> Workbook: ...
    @staticmethod
    def from_bytes(
        data: bytes,
        recalc: bool = True,
        clock: int | None = None,
        name: str | None = None,
        password: str | None = None,
    ) -> Workbook: ...
    def save_xlsx(self, path: str) -> None: ...
    def to_xlsx_bytes(self) -> bytes: ...
    def recalculate(self) -> None: ...
    @property
    def iteration(self) -> tuple[int, float] | None:
        """Iterative calculation as (max_iterations, max_change); None when off."""
    def set_iteration(
        self, enabled: bool = True, max_iterations: int = 100, max_change: float = 0.001
    ) -> None:
        """Turn iterative calculation on or off; applies from the next recalculation.

        max_iterations must be 1..=32767 and max_change finite and >= 0, else ValueError.
        """
    @property
    def sheet_names(self) -> list[str]: ...
    def value(self, ref: str) -> CellValue: ...
    def __getitem__(self, ref: str) -> CellValue: ...
    def values(self, ref: str) -> list[list[CellValue]]: ...
    def formula(self, ref: str) -> str | None: ...
    def number_format(self, ref: str) -> str | None: ...
    def formatted(self, ref: str) -> str: ...
    def used_range(self, sheet: str) -> str | None: ...
    def set_track_changes(self, on: bool) -> None: ...
    def set(self, ref: str, value: EditValue) -> None: ...
    def edit(self) -> Transaction: ...
    @property
    def changes(self) -> dict[str, list[dict[str, Any]]] | None: ...

def column_name_from_number(column: int) -> str: ...
def column_number_from_name(column: str) -> int: ...
def quote_name(name: str) -> str: ...
def get_all_timezones() -> list[str]: ...
def get_supported_locales() -> list[str]: ...
