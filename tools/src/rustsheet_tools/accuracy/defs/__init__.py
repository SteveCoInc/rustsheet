"""Gate A definitions by family; importing this package registers every definition.

Family modules are discovered, not listed: adding a family means adding one module here, with no
edit to a shared file (T-3830). Discovery is sorted, and an import failure propagates.
"""

import importlib
import pkgutil

__all__ = [
    name for name in sorted(i.name for i in pkgutil.iter_modules(__path__)) if not name.startswith("_")
]

for _name in __all__:
    importlib.import_module(f"{__name__}.{_name}")
