"""The deprecated legacy API: :class:`UserModel` and :class:`Model`.

These classes carry features the facade (:class:`rustsheet.Workbook`) does not have yet: styles and
named styles, undo/redo, conditional formatting, the clipboard, links, frozen panes, and row and
column sizes. They go at the API v1 freeze (SPEC-0026), once each feature has moved to the facade
or been dropped.

They are not hardened for untrusted input: their loaders skip the facade's load limits
(``LoadOptions``), and ``UserModel.paste_from_clipboard`` and ``UserModel.paste_csv_string`` take
caller-shaped data. Use :class:`rustsheet.Workbook` for workbooks you did not write.

* The **user API** (:class:`UserModel`): every action evaluates the workbook and keeps undo/redo
  history.
* The **raw API** (:class:`Model`): nothing is evaluated automatically (call
  :meth:`Model.evaluate` yourself), there is no undo/redo. Faster, but easier to get wrong.
"""

import warnings

from rustsheet._rustsheet import (
    CellType,
    Model,
    UserModel,
    create,
    create_user_model,
    create_user_model_from_xlsx,
    load_from_xlsx,
)

# stacklevel=2 names the importing module: `warnings` skips importlib's own frames.
warnings.warn(
    "rustsheet.legacy (Model, UserModel and their constructors) is deprecated and is removed at "
    "the API v1 freeze; it skips the load limits of rustsheet.Workbook, the supported API",
    DeprecationWarning,
    stacklevel=2,
)

__all__ = [
    "CellType",
    "Model",
    "UserModel",
    "create",
    "create_user_model",
    "create_user_model_from_xlsx",
    "load_from_xlsx",
]
