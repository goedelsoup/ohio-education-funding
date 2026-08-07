"""Build the committed CSV fixtures the Rust crates read.

Each builder is a pure function from parsed rows to rows-of-strings, so it can be tested
without a network or a spreadsheet. :func:`write_all` is the only part that touches disk.

The fixtures are the boundary between the two halves of the domain computer: Python does
retrieval and extraction because the ecosystem is there, Rust does calculation because it must
be deterministic and auditable. A CSV in between is legible in a diff, which a binary would
not be.
"""

from __future__ import annotations

import csv
from pathlib import Path
from typing import Iterable

from .xlsx import Workbook, number, rows_by_key

REPO = Path(__file__).resolve().parents[3]

#: The aggregate row the department includes in Summary_SFPR. Its IRN is numeric, so it
#: survives the digit filter and must be excluded by name. Counting it as a district
#: double-counts every statewide total.
STATEWIDE_ROW = "State of Ohio"


def _fmt(v: float | None, places: int = 4) -> str:
    if v is None:
        return ""
    return f"{v:.{places}f}".rstrip("0").rstrip(".")


def _clean_name(v: str) -> str:
    """District names go into a comma-separated file, so strip commas rather than quote."""
    return str(v).replace(",", " ").strip()


FY27_HEADER = [
    "irn", "district", "base_cost_enrolled_adm", "school_buildings",
    "adm_kindergarten", "adm_grades_1_3", "adm_grades_4_8_non_cte",
    "adm_grades_9_12_non_cte", "adm_cte", "adm_grades_9_12_total",
    "funded_classroom_teachers", "funded_special_teachers", "teacher_base_cost",
    "aggregate_base_cost", "base_cost_per_pupil", "temp_transitional_aid_guarantee",
    "enrolled_adm_fy22", "enrolled_adm_fy24", "assessed_valuation_per_pupil_fy23",
    "core_foundation_funding",
]

# Column positions in the department's Base_Cost sheet (header on the fourth row) and
# Summary_SFPR sheet. Named here rather than inline so a layout change is one edit.
_BC = dict(
    buildings=3, adm_fy22=4, adm_fy24=6, adm=7, kdg=8, g13=9, g48=10, g912=11,
    cte=12, g912_total=13, funded_classroom=14, funded_special=17, teacher_cost=22,
    aggregate=57, per_pupil=58,
)
_SM = dict(core=10, guarantee=11)
_CUPP_VALUATION = 21


def build_fy27_model(
    base_cost_rows: list[list[str]],
    summary_rows: list[list[str]],
    cupp_rows: list[list[str]],
) -> list[list[str]]:
    """Join the department's FY2027 base cost and summary sheets with Cupp valuation."""
    summary = {
        irn: row
        for irn, row in rows_by_key(summary_rows)
        if STATEWIDE_ROW not in str(row[1] if len(row) > 1 else "")
    }
    valuation = {
        str(row[1]).strip(): row for row in cupp_rows[1:] if len(row) > 1
    }

    out: list[list[str]] = []
    for irn, bc in sorted(rows_by_key(base_cost_rows)):
        sm = summary.get(irn)
        if sm is None:
            continue
        adm = number(bc[_BC["adm"]] if len(bc) > _BC["adm"] else None)
        if not adm or adm <= 0:
            continue

        def bcv(k: str) -> float | None:
            i = _BC[k]
            return number(bc[i]) if len(bc) > i else None

        cupp = valuation.get(irn)
        val = (
            number(cupp[_CUPP_VALUATION])
            if cupp and len(cupp) > _CUPP_VALUATION
            else None
        )
        out.append([
            irn,
            _clean_name(bc[1] if len(bc) > 1 else ""),
            _fmt(adm), _fmt(bcv("buildings"), 0),
            _fmt(bcv("kdg")), _fmt(bcv("g13")), _fmt(bcv("g48")),
            _fmt(bcv("g912")), _fmt(bcv("cte")), _fmt(bcv("g912_total")),
            _fmt(bcv("funded_classroom"), 2), _fmt(bcv("funded_special"), 2),
            _fmt(bcv("teacher_cost"), 2), _fmt(bcv("aggregate"), 2),
            _fmt(bcv("per_pupil"), 2),
            _fmt(number(sm[_SM["guarantee"]]) or 0.0, 2),
            _fmt(bcv("adm_fy22")), _fmt(bcv("adm_fy24")),
            _fmt(val, 2),
            _fmt(number(sm[_SM["core"]]), 2),
        ])
    return out


CUPP_HEADER = [
    "irn", "district", "enrolled_adm_fy24", "econ_disadvantaged_pct_fy24",
    "assessed_valuation_per_pupil_fy23", "current_operating_millage_ty23",
    "effective_class1_millage_ty23", "operating_expenditure_per_pupil_fy24",
    "state_revenue_per_pupil_fy24", "local_revenue_per_pupil_fy24",
]

_CUPP_KEEP = [(1, "irn"), (0, "district"), (4, None), (11, None), (21, None),
              (33, None), (34, None), (46, None), (47, None), (49, None)]


def build_cupp_extract(cupp_rows: list[list[str]]) -> list[list[str]]:
    """Reduce the District Profile Report's 60 columns to the ten the corpus uses."""
    out: list[list[str]] = []
    for row in cupp_rows[1:]:
        if len(row) < 2 or not str(row[1]).strip():
            continue
        rec: list[str] = []
        for idx, kind in _CUPP_KEEP:
            raw = row[idx] if idx < len(row) else ""
            if kind == "irn":
                rec.append(str(raw).strip())
            elif kind == "district":
                rec.append(_clean_name(raw))
            else:
                rec.append(_fmt(number(raw)))
        out.append(rec)
    return out


def write_csv(path: Path, header: Iterable[str], rows: Iterable[Iterable[str]]) -> int:
    """Write a fixture with newline='' and LF endings, so git sees no spurious churn."""
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("w", newline="") as fh:
        w = csv.writer(fh, lineterminator="\n")
        w.writerow(list(header))
        n = 0
        for r in rows:
            w.writerow(list(r))
            n += 1
    return n


def write_all(fy27: Workbook, cupp: Workbook, repo: Path | None = None) -> dict[str, int]:
    """Rebuild every committed fixture. Returns row counts by path."""
    root = Path(repo) if repo else REPO
    cupp_rows = cupp.rows("District Data")
    counts: dict[str, int] = {}

    counts["crates/foundation/fixtures/fy27-department-model.csv"] = write_csv(
        root / "crates/foundation/fixtures/fy27-department-model.csv",
        FY27_HEADER,
        build_fy27_model(fy27.rows("Base_Cost"), fy27.rows("Summary_SFPR"), cupp_rows),
    )
    counts["crates/dispersion/fixtures/cupp-fy24-district-data.csv"] = write_csv(
        root / "crates/dispersion/fixtures/cupp-fy24-district-data.csv",
        CUPP_HEADER,
        build_cupp_extract(cupp_rows),
    )
    return counts
