# RustSheet Python bindings

`import rustsheet` gives you a headless, Excel-compatible spreadsheet engine: open, edit, evaluate
and save xlsx workbooks from Python, with Excel's values, errors and number formats.

## Installation

The package is not on PyPI yet. Build it from this directory with
[maturin](https://www.maturin.rs/) (see "Compile and test" below):

```bash
pip install maturin
maturin develop
```

## Quick start

The API is `rustsheet.Workbook`. Cells are addressed by A1 references, values are
`None | float | str | bool | XlError`, and edits are atomic transactions:

```python
import rustsheet as rs

wb = rs.Workbook()

# Edits apply on a clean exit of the block and are discarded if it raises.
with wb.edit() as tx:
    tx["Sheet1!A1"] = 10
    tx["Sheet1!A2"] = rs.Formula("=A1*2")
    tx["Sheet1!A3"] = "=1/0"

print(wb["Sheet1!A2"])            # 20.0
print(wb["Sheet1!A3"])            # #DIV/0!  (an rs.XlError value, not an exception)
print(wb.formatted("Sheet1!A2"))  # "20"
print(wb.changes["values"])       # the cells whose value changed in the last edit

wb.save_xlsx("my-workbook.xlsx")
```

Open an existing file with `rs.open("book.xlsx")` (or `rs.Workbook.from_bytes(data)` for bytes in
memory). Loading enforces resource limits, so the facade is the API to use for workbooks you did
not write. Failures raise the typed errors `LoadError`, `SaveError`, `EditError` and `RefError`
(all `RustSheetError`). `docs/examples/facade.py` is a runnable copy of the quick start.

A password-protected xlsx (ECMA-376 Agile or Standard encryption) opens with
`rs.open("book.xlsx", password="...")`; `Workbook.open` and `Workbook.from_bytes` take the same
`password=` keyword. Without a password it raises `PasswordRequiredError`, with the wrong one
`WrongPasswordError` (both subclasses of `LoadError`, so `except rs.LoadError` still catches them).
A legacy `.xls` raises `LoadError` and should be saved as xlsx first. The password never appears
in an error message and is not kept on the workbook: the Rust side copies it into a buffer it
zeroes after the load, but the Python `str` you passed is an ordinary immutable string that Python
cannot wipe. Saving such a workbook writes an unencrypted file.

## The legacy module (deprecated)

`rustsheet.legacy` holds the older index-based classes, `UserModel` and `Model`, with `CellType`
and the constructors `create`, `create_user_model`, `load_from_xlsx` and
`create_user_model_from_xlsx`. They carry features the facade does not have yet (styles and named
styles, undo/redo, conditional formatting, the clipboard, links, frozen panes, row and column
sizes). Importing the module raises a `DeprecationWarning`: it is removed at the API v1 freeze,
once each of those features has moved to the facade or been dropped.

The legacy module is not hardened for untrusted input. Its loaders skip the facade's load limits,
and `UserModel.paste_from_clipboard` and `UserModel.paste_csv_string` take caller-shaped data.

```python
from rustsheet import legacy

model = legacy.UserModel("my-workbook")
model.set_user_input(0, 1, 1, "150")  # sheet index, row, column (1-based)
model.update_range_style(0, 1, 1, 1, 1, "font.b", "true")
model.save_to_xlsx("styled.xlsx")
```

The other examples in `docs/examples` (styled report, conditional formatting, CSV import) use it.

## Removed legacy surface (pre-v1, no warning release)

The bitcode/icalc loaders and writers (`load_from_bytes`, `load_from_icalc`,
`create_user_model_from_bytes`, `create_user_model_from_icalc`, `UserModel.from_bytes`,
`UserModel.from_icalc`, `save_to_icalc`, `to_bytes`), the diff exchange
(`UserModel.apply_external_diffs`, `UserModel.flush_send_queue`) and `test_panic` are gone
(SPEC-0026). Use xlsx (`rustsheet.open`, `Workbook.from_bytes`, `save_xlsx`).

## Compile and test

```bash
./run_tests.sh      # build with maturin and run the pytest suite
./run_examples.sh   # run all the examples in docs/examples
```

Or manually:

```bash
python3 -m venv venv
source venv/bin/activate
pip install maturin pytest
maturin develop
pytest tests/
```

## Creating documentation

We use sphinx:

```bash
python -m venv venv
source venv/bin/activate
pip install maturin sphinx
maturin develop
sphinx-build -M html docs html
python -m http.server --directory html/html/
```
