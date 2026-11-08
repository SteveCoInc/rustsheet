
Usage Examples
--------------

All the examples on this page live in ``docs/examples`` in the repository and
are run as part of the test suite, so they always work with the current
version.

The facade
^^^^^^^^^^

:class:`rustsheet.Workbook` is the API for new code. Loading enforces resource limits, Excel
errors come back as :class:`rustsheet.XlError` values and every edit is a transaction:

.. literalinclude:: examples/facade.py
   :language: python

The legacy module (deprecated)
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

``rustsheet.legacy`` holds the older index-based classes, kept until the API v1 freeze for the
features the facade does not have yet (styles, undo/redo, conditional formatting, the clipboard).
Importing it raises a ``DeprecationWarning``, and it is not hardened for untrusted input.
The remaining examples on this page use it.

* The **user API** (``legacy.UserModel``): every action evaluates the workbook and keeps
  undo/redo history.
* The **raw API** (``legacy.Model``): a low level API. Nothing is evaluated automatically
  (call :meth:`Model.evaluate` yourself) and there is no undo/redo.

.. literalinclude:: examples/simple.py
   :language: python

A styled report
^^^^^^^^^^^^^^^

Building a small sales report: bold headers, background colors, currency
formats, borders and column widths, saved as xlsx.

.. literalinclude:: examples/styled_report.py
   :language: python

Conditional formatting
^^^^^^^^^^^^^^^^^^^^^^

Color scales and cell-is rules. Rules are plain dictionaries (see :doc:`objects`).

.. literalinclude:: examples/conditional_formatting.py
   :language: python

Reading an xlsx file
^^^^^^^^^^^^^^^^^^^^

Values come back as native Python types (``None``, ``str``, ``float`` or
``bool``).

.. literalinclude:: examples/read_xlsx.py
   :language: python

Importing CSV data
^^^^^^^^^^^^^^^^^^

.. literalinclude:: examples/csv_import.py
   :language: python
