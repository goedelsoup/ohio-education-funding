"""Retrieval and extraction for the Ohio education funding corpus.

The Python half of the domain computer. It fetches the department's publications, parses
them, and writes the CSV fixtures the Rust crates compute against. Python is here for
ecosystem reasons — XLSX and legacy spreadsheet handling — not because the logic belongs in
a dynamic language; everything that has to be deterministic and auditable lives in
``crates/``.

Standard library only, so it runs anywhere Python does with no environment to build.

Typical use::

    python3 -m ohio_edfund_extract --list
    python3 -m ohio_edfund_extract --rebuild
"""

from .sources import SOURCES, RetrievalError, Source, ensure_readable, fetch
from .xlsx import Workbook, number, open_workbook, rows_by_key

__all__ = [
    "SOURCES",
    "RetrievalError",
    "Source",
    "Workbook",
    "ensure_readable",
    "fetch",
    "number",
    "open_workbook",
    "rows_by_key",
]

__version__ = "0.1.0"
