"""Tests for the stdlib XLSX reader.

Workbooks are built in-memory rather than committed as binary fixtures, so the tests are
hermetic and the expected structure is visible in the test itself.
"""

from __future__ import annotations

import unittest
import zipfile
from pathlib import Path
from tempfile import TemporaryDirectory

from ohio_edfund_extract.xlsx import (
    column_index,
    number,
    open_workbook,
    rows_by_key,
)

_RELS = """<?xml version="1.0"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
<Relationship Id="rId1" Target="worksheets/sheet1.xml"/>
<Relationship Id="rId2" Target="worksheets/sheet2.xml"/>
</Relationships>"""

_WORKBOOK = """<?xml version="1.0"?>
<workbook xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main"
 xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships">
<sheets>
<sheet name="Notes" sheetId="1" r:id="rId1"/>
<sheet name="District Data" sheetId="2" r:id="rId2"/>
</sheets></workbook>"""

_SHARED = """<?xml version="1.0"?>
<sst xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main" count="3" uniqueCount="3">
<si><t>IRN</t></si><si><t>District</t></si><si><t>State of Ohio</t></si>
</sst>"""

_NOTES = """<?xml version="1.0"?>
<worksheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main"><sheetData>
<row r="1"><c r="A1" t="inlineStr"><is><t>a note</t></is></c></row>
</sheetData></worksheet>"""

# Row 2 is deliberately empty (a spacer). Row 5 is the statewide aggregate.
_DATA = """<?xml version="1.0"?>
<worksheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main"><sheetData>
<row r="1"><c r="A1" t="s"><v>0</v></c><c r="B1" t="s"><v>1</v></c></row>
<row r="2"/>
<row r="3"><c r="A3"><v>043786</v></c><c r="B3" t="inlineStr"><is><t>Cleveland</t></is></c>
  <c r="D3"><v>184903.95</v></c></row>
<row r="4"><c r="A4"><v>049056</v></c><c r="B4" t="inlineStr"><is><t>Northern</t></is></c>
  <c r="D4" t="inlineStr"><is><t>#N/A</t></is></c></row>
<row r="5"><c r="A5"><v>999999</v></c><c r="B5" t="s"><v>2</v></c></row>
</sheetData></worksheet>"""


def make_workbook(path: Path) -> None:
    with zipfile.ZipFile(path, "w") as z:
        z.writestr("xl/_rels/workbook.xml.rels", _RELS)
        z.writestr("xl/workbook.xml", _WORKBOOK)
        z.writestr("xl/sharedStrings.xml", _SHARED)
        z.writestr("xl/worksheets/sheet1.xml", _NOTES)
        z.writestr("xl/worksheets/sheet2.xml", _DATA)


class ColumnIndexTests(unittest.TestCase):
    def test_single_and_double_letter_columns(self):
        self.assertEqual(column_index("A1"), 0)
        self.assertEqual(column_index("D3"), 3)
        self.assertEqual(column_index("Z9"), 25)
        self.assertEqual(column_index("AA1"), 26)
        self.assertEqual(column_index("BA12"), 52)

    def test_rejects_a_non_reference(self):
        with self.assertRaises(ValueError):
            column_index("123")


class NumberTests(unittest.TestCase):
    def test_parses_plain_and_formatted_numbers(self):
        self.assertEqual(number("1234.5"), 1234.5)
        self.assertEqual(number(" 1,234.50 "), 1234.5)
        self.assertEqual(number("$8,241.61"), 8241.61)

    def test_department_missing_markers_become_none(self):
        for marker in ("", "#N/A", "#DIV/0!", "   "):
            self.assertIsNone(number(marker), marker)

    def test_suppressed_small_counts_are_none_not_zero(self):
        # "<10" means "fewer than ten, withheld". Treating it as 0 understates any
        # aggregate over small districts, so it must not parse to a number.
        self.assertIsNone(number("<10"))

    def test_unparseable_text_is_none_rather_than_an_exception(self):
        self.assertIsNone(number("not a number"))


class WorkbookTests(unittest.TestCase):
    def setUp(self):
        self._tmp = TemporaryDirectory()
        self.path = Path(self._tmp.name) / "book.xlsx"
        make_workbook(self.path)

    def tearDown(self):
        self._tmp.cleanup()

    def test_lists_sheets_in_workbook_order(self):
        with open_workbook(str(self.path)) as wb:
            self.assertEqual(wb.sheet_names, ["Notes", "District Data"])

    def test_resolves_shared_strings_and_inline_strings(self):
        with open_workbook(str(self.path)) as wb:
            rows = wb.rows("District Data")
            self.assertEqual(rows[0][:2], ["IRN", "District"])
            self.assertEqual(rows[1][1], "Cleveland")

    def test_skips_empty_spacer_rows(self):
        with open_workbook(str(self.path)) as wb:
            rows = wb.rows("District Data")
        # Header, two districts, one aggregate — the blank row 2 is dropped.
        self.assertEqual(len(rows), 4)

    def test_pads_short_rows_to_the_widest_cell(self):
        with open_workbook(str(self.path)) as wb:
            row = wb.rows("District Data")[1]
        self.assertEqual(len(row), 4)
        self.assertEqual(row[2], "")
        self.assertEqual(row[3], "184903.95")

    def test_limit_stops_early(self):
        with open_workbook(str(self.path)) as wb:
            self.assertEqual(len(wb.rows("District Data", limit=2)), 2)

    def test_unknown_sheet_names_the_available_ones(self):
        with open_workbook(str(self.path)) as wb:
            with self.assertRaises(KeyError) as ctx:
                wb.rows("Nope")
        self.assertIn("District Data", str(ctx.exception))


class RowsByKeyTests(unittest.TestCase):
    def setUp(self):
        self._tmp = TemporaryDirectory()
        self.path = Path(self._tmp.name) / "book.xlsx"
        make_workbook(self.path)

    def tearDown(self):
        self._tmp.cleanup()

    def test_drops_the_header_banner_but_keeps_the_aggregate_row(self):
        with open_workbook(str(self.path)) as wb:
            keyed = list(rows_by_key(wb.rows("District Data")))
        keys = [k for k, _ in keyed]
        self.assertEqual(keys, ["043786", "049056", "999999"])
        # The statewide aggregate has a numeric key and survives the filter by design —
        # excluding it is the caller's job, and forgetting to double-counts every total.
        self.assertIn("State of Ohio", keyed[-1][1])


if __name__ == "__main__":
    unittest.main()
