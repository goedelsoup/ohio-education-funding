"""Tests for the fixture builders.

The builders are pure functions from parsed rows to rows-of-strings, so these run without a
network, a spreadsheet, or the real 5 MB workbooks.
"""

from __future__ import annotations

import csv
import unittest
from pathlib import Path
from tempfile import TemporaryDirectory

from ohio_edfund_extract.fixtures import (
    CUPP_HEADER,
    FY27_HEADER,
    STATEWIDE_ROW,
    build_cupp_extract,
    build_fy27_model,
    write_csv,
)


def base_cost_rows() -> list[list[str]]:
    """Three banner rows, a header, then two districts — the department's layout."""
    banner = [["Ohio Department of Education & Workforce"], [""], ["Base Cost"]]
    header = ["District IRN", "District Names", "County"] + [f"c{i}" for i in range(3, 59)]
    def district(irn: str, name: str) -> list[str]:
        row = [""] * 59
        row[0], row[1], row[2] = irn, name, "Somewhere"
        row[3] = "12"          # buildings
        row[4], row[6] = "2200", "2100"   # FY22 / FY24 enrolled ADM
        row[7] = "2150.5"      # base cost enrolled ADM
        row[8], row[9], row[10], row[11] = "150", "450", "760", "700"
        row[12], row[13] = "90", "790"
        row[14], row[17] = "84.21", "14.34"
        row[22] = "9500000.55"
        row[57], row[58] = "17400000.25", "8091.16"
        return row
    return banner + [header, district("043786", "Cleveland"), district("049056", "Northern")]


def summary_rows() -> list[list[str]]:
    header = ["District IRN", "District Names"] + [f"c{i}" for i in range(2, 12)]
    def row(irn: str, name: str, core: str, guarantee: str) -> list[str]:
        r = [""] * 12
        r[0], r[1], r[10], r[11] = irn, name, core, guarantee
        return r
    return [
        header,
        row("043786", "Cleveland", "10000000", "0"),
        row("049056", "Northern", "7000000", "2500000"),
        # The aggregate the department ships in this sheet. Numeric IRN; must be excluded.
        row("999999", STATEWIDE_ROW, "999999999", "888888888"),
    ]


def cupp_rows() -> list[list[str]]:
    header = [f"c{i}" for i in range(50)]
    header[0], header[1] = "District", "IRN"
    def row(irn: str, name: str, valuation: str) -> list[str]:
        r = [""] * 50
        r[0], r[1] = name, irn
        r[4], r[11], r[21] = "2100", "0.42", valuation
        r[33], r[34], r[46], r[47], r[49] = "31.35", "20", "11986.62", "6423.28", "5277.43"
        return r
    return [header, row("043786", "Cleveland Municipal", "184903.95"),
            row("049056", "Northern Local", "279983.24")]


class Fy27ModelTests(unittest.TestCase):
    def setUp(self):
        self.rows = build_fy27_model(base_cost_rows(), summary_rows(), cupp_rows())

    def test_emits_one_row_per_district(self):
        self.assertEqual(len(self.rows), 2)
        self.assertEqual([r[0] for r in self.rows], ["043786", "049056"])

    def test_excludes_the_statewide_aggregate_row(self):
        # Its IRN is numeric so the digit filter keeps it; only the name check drops it.
        self.assertNotIn("999999", [r[0] for r in self.rows])

    def test_row_width_matches_the_declared_header(self):
        for r in self.rows:
            self.assertEqual(len(r), len(FY27_HEADER))

    def test_joins_valuation_from_the_profile_report(self):
        northern = next(r for r in self.rows if r[0] == "049056")
        self.assertEqual(northern[FY27_HEADER.index("assessed_valuation_per_pupil_fy23")],
                         "279983.24")

    def test_carries_the_guarantee_and_core_funding(self):
        northern = next(r for r in self.rows if r[0] == "049056")
        self.assertEqual(
            northern[FY27_HEADER.index("temp_transitional_aid_guarantee")], "2500000")
        self.assertEqual(northern[FY27_HEADER.index("core_foundation_funding")], "7000000")

    def test_a_district_with_no_guarantee_records_zero_not_blank(self):
        # Blank would read as "unknown"; this district is known to have none.
        cleveland = next(r for r in self.rows if r[0] == "043786")
        self.assertEqual(cleveland[FY27_HEADER.index("temp_transitional_aid_guarantee")], "0")

    def test_skips_a_district_missing_from_the_summary_sheet(self):
        rows = build_fy27_model(base_cost_rows(), [summary_rows()[0]], cupp_rows())
        self.assertEqual(rows, [])

    def test_skips_a_district_with_no_usable_adm(self):
        bc = base_cost_rows()
        bc[-1][7] = "0"
        rows = build_fy27_model(bc, summary_rows(), cupp_rows())
        self.assertEqual([r[0] for r in rows], ["043786"])

    def test_commas_are_stripped_from_district_names(self):
        bc = base_cost_rows()
        bc[-1][1] = "Northern Local, Perry"
        rows = build_fy27_model(bc, summary_rows(), cupp_rows())
        self.assertNotIn(",", rows[-1][1])


class CuppExtractTests(unittest.TestCase):
    def setUp(self):
        self.rows = build_cupp_extract(cupp_rows())

    def test_reduces_to_the_declared_columns(self):
        self.assertEqual(len(self.rows), 2)
        for r in self.rows:
            self.assertEqual(len(r), len(CUPP_HEADER))

    def test_puts_irn_first_and_name_second(self):
        self.assertEqual(self.rows[0][0], "043786")
        self.assertEqual(self.rows[0][1], "Cleveland Municipal")

    def test_missing_values_are_blank_not_zero(self):
        rows = cupp_rows()
        rows[1][21] = "#N/A"
        out = build_cupp_extract(rows)
        self.assertEqual(out[0][CUPP_HEADER.index("assessed_valuation_per_pupil_fy23")], "")


class WriteCsvTests(unittest.TestCase):
    def test_writes_lf_endings_and_returns_a_row_count(self):
        with TemporaryDirectory() as tmp:
            p = Path(tmp) / "nested" / "out.csv"
            n = write_csv(p, ["a", "b"], [["1", "2"], ["3", "4"]])
            self.assertEqual(n, 2)
            raw = p.read_bytes()
            self.assertNotIn(b"\r\n", raw, "CRLF would churn the diff on every rebuild")
            with p.open() as fh:
                self.assertEqual(list(csv.reader(fh))[0], ["a", "b"])


if __name__ == "__main__":
    unittest.main()
