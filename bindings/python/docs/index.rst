RustSheet
=========

.. toctree::
   :maxdepth: 2
   :caption: Contents:

   installation
   usage_examples
   facade_api_reference
   top_level_methods
   raw_api_reference
   user_api_reference
   objects

RustSheet is a headless, Excel-compatible spreadsheet engine: open, edit, evaluate and save
xlsx workbooks from Python, with Excel's values, errors and number formats.

The API is :class:`rustsheet.Workbook`. Cells are addressed by A1 references, values are
``None | float | str | bool | XlError``, and edits are atomic transactions:

.. literalinclude:: examples/facade.py
   :language: python

The older index-based classes ``UserModel`` and ``Model`` live in the deprecated
``rustsheet.legacy`` module until the API v1 freeze; the last three pages document them.

RustSheet's engine started as a fork of IronCalc.
