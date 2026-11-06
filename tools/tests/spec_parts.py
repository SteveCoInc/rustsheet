"""The one helper the spec2xlsx tests share: build a spec and read its XML parts."""

import io
import zipfile
from typing import Any

from rustsheet_tools.spec2xlsx import build_xlsx


def spec_parts(spec: dict[str, Any], suffixes: tuple[str, ...] = (".xml",)) -> dict[str, str]:
    """Build `spec` with `build_xlsx` and return its package parts ending in `suffixes`, decoded.

    `suffixes` defaults to the XML parts; pass `(".xml", ".rels")` to read relationships too.
    """
    z = zipfile.ZipFile(io.BytesIO(build_xlsx(spec)))
    return {n: z.read(n).decode() for n in z.namelist() if n.endswith(suffixes)}
