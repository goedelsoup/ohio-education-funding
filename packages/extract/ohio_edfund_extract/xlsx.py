"""Read XLSX workbooks with nothing but the standard library.

An XLSX is a zip of XML, so ``zipfile`` plus ``xml.etree`` is sufficient. That matters here
because it keeps the extraction layer installable anywhere Python runs, with no wheel to
resolve — the same property the Rust workspace maintains by having no crates.io dependencies.

Cached formula results live in the ``<v>`` element, so a workbook full of formulas reads
without a spreadsheet engine. That is what makes the department's own funding calculator
usable as a data source rather than only as a document.

Legacy ``.xls`` (OLE2) is a different format entirely and this module does not read it. See
:func:`ohio_edfund_extract.sources.convert_legacy_xls`.
"""

from __future__ import annotations

import re
import zipfile
import xml.etree.ElementTree as ET
from dataclasses import dataclass, field
from typing import Iterator

_MAIN = "http://schemas.openxmlformats.org/spreadsheetml/2006/main"
_REL = "http://schemas.openxmlformats.org/officeDocument/2006/relationships"

_COL = re.compile(r"([A-Z]+)")


def column_index(cell_ref: str) -> int:
    """Zero-based column index from a cell reference such as ``BA12``.

    >>> column_index("A1"), column_index("Z9"), column_index("AA1")
    (0, 25, 26)
    """
    letters = _COL.match(cell_ref)
    if letters is None:
        raise ValueError(f"not a cell reference: {cell_ref!r}")
    n = 0
    for ch in letters.group(1):
        n = n * 26 + (ord(ch) - 64)
    return n - 1


@dataclass
class Workbook:
    """An opened XLSX, with its shared-string table and sheet name map resolved."""

    path: str
    _zip: zipfile.ZipFile = field(repr=False)
    _shared: list[str] = field(repr=False)
    _sheets: dict[str, str] = field(repr=False)

    @property
    def sheet_names(self) -> list[str]:
        """Sheet names in workbook order."""
        return list(self._sheets)

    def rows(self, sheet: str, limit: int | None = None) -> list[list[str]]:
        """All rows of ``sheet`` as lists of strings, padded to the widest cell present.

        Empty cells become empty strings. Rows with no populated cells are skipped, which is
        what the department's workbooks use as visual spacing.
        """
        if sheet not in self._sheets:
            raise KeyError(f"no sheet {sheet!r}; have {self.sheet_names}")
        target = self._sheets[sheet]
        path = "xl/" + target.lstrip("/").removeprefix("xl/")
        root = ET.fromstring(self._zip.read(path))

        out: list[list[str]] = []
        for row in root.iter(f"{{{_MAIN}}}row"):
            cells: dict[int, str] = {}
            for c in row.iter(f"{{{_MAIN}}}c"):
                value = self._cell_value(c)
                if value is None:
                    continue
                cells[column_index(c.get("r", "A1"))] = value
            if cells:
                width = max(cells) + 1
                out.append([cells.get(i, "") for i in range(width)])
            if limit is not None and len(out) >= limit:
                break
        return out

    def _cell_value(self, c: ET.Element) -> str | None:
        kind = c.get("t")
        if kind == "inlineStr":
            inline = c.find(f"{{{_MAIN}}}is")
            if inline is None:
                return None
            return "".join(t.text or "" for t in inline.iter(f"{{{_MAIN}}}t"))
        v = c.find(f"{{{_MAIN}}}v")
        if v is None or v.text is None:
            return None
        if kind == "s":
            return self._shared[int(v.text)]
        return v.text

    def close(self) -> None:
        self._zip.close()

    def __enter__(self) -> "Workbook":
        return self

    def __exit__(self, *_exc: object) -> None:
        self.close()


def open_workbook(path: str) -> Workbook:
    """Open an XLSX file."""
    z = zipfile.ZipFile(path)

    shared: list[str] = []
    if "xl/sharedStrings.xml" in z.namelist():
        sst = ET.fromstring(z.read("xl/sharedStrings.xml"))
        for si in sst.iter(f"{{{_MAIN}}}si"):
            shared.append("".join(t.text or "" for t in si.iter(f"{{{_MAIN}}}t")))

    rels = {
        r.get("Id"): r.get("Target")
        for r in ET.fromstring(z.read("xl/_rels/workbook.xml.rels"))
    }
    wb = ET.fromstring(z.read("xl/workbook.xml"))
    sheets: dict[str, str] = {}
    for s in wb.iter(f"{{{_MAIN}}}sheet"):
        name = s.get("name")
        rid = s.get(f"{{{_REL}}}id")
        if name is not None and rid in rels:
            sheets[name] = rels[rid] or ""
    return Workbook(path=path, _zip=z, _shared=shared, _sheets=sheets)


# --- reading conventions specific to the department's publications -----------------------

#: Values the department uses for "no data". ``<10`` is a suppressed small count and is NOT
#: zero — treating it as zero understates aggregates over small districts.
MISSING = frozenset({"", "#N/A", "#DIV/0!", "#VALUE!", "#REF!"})
SUPPRESSED = "<10"


def number(cell: str | None) -> float | None:
    """Parse a department cell into a float, or ``None`` where there is no usable value.

    Suppressed small counts (``<10``) return ``None`` rather than 0.0. A caller that wants to
    treat them as zero must say so explicitly, because the two are not the same claim.
    """
    if cell is None:
        return None
    s = cell.strip()
    if s in MISSING or s == SUPPRESSED:
        return None
    try:
        return float(s.replace(",", "").replace("$", ""))
    except ValueError:
        return None


def rows_by_key(rows: list[list[str]], key_column: int = 0) -> Iterator[tuple[str, list[str]]]:
    """Yield ``(key, row)`` for rows whose key column holds a digit string.

    The department's per-district sheets carry title banners, blank spacers, and — in
    ``Summary_SFPR`` — a ``State of Ohio`` aggregate row. Filtering on a numeric IRN drops the
    banners; the aggregate row must be excluded by the caller, since its key is numeric too.
    """
    for row in rows:
        if not row:
            continue
        key = str(row[key_column]).strip() if key_column < len(row) else ""
        if key.isdigit():
            yield key, row
