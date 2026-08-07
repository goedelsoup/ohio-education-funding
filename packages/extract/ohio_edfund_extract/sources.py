"""The source registry: where each committed fixture comes from, and how to fetch it.

Every entry here corresponds to a node in [`.yidam/catalog/`](../../../.yidam/catalog/). The
catalog says what a source is and what it can be trusted for; this module says how to get it.
Keeping them separate means a URL change is an operational commit and a provenance change is
an epistemic one.

Downloads are cached by filename so a re-run is cheap and offline work is possible after one
fetch — the offline-mode requirement the prelude places on connectors.
"""

from __future__ import annotations

import shutil
import subprocess
import urllib.request
from dataclasses import dataclass
from pathlib import Path

USER_AGENT = "ohio-education-funding-corpus/0.1 (research; contact via repository)"

DEFAULT_CACHE = Path(__file__).resolve().parents[3] / ".cache" / "sources"


@dataclass(frozen=True)
class Source:
    """One retrievable publication."""

    key: str
    url: str
    filename: str
    catalog: str
    """Slug of the `.yidam/catalog/` node describing this source."""
    note: str = ""
    legacy_xls: bool = False
    """True for OLE2 ``.xls``, which must be converted before :mod:`.xlsx` can read it."""


SOURCES: dict[str, Source] = {
    "fy27-calculator": Source(
        key="fy27-calculator",
        url=(
            "https://education.ohio.gov/getattachment/Topics/Finance-and-Funding/"
            "School-Payment-Reports/State-Funding-For-Schools/Traditional-School-Districts/"
            "FY27-TRAD-State-Foundation-Funding-Calculator_12-16-2025_lock-1.xlsx.aspx"
            "?lang=en-US"
        ),
        filename="fy27-calculator.xlsx",
        catalog="dew-fy27-funding-calculator",
        note="The department's own FY2027 model. A projection, not an actual.",
    ),
    "cupp-fy24": Source(
        key="cupp-fy24",
        url=(
            "https://education.ohio.gov/getattachment/Topics/Finance-and-Funding/"
            "School-Payment-Reports/District-Profile-Reports/FY2024-District-Profile-Report/"
            "FY24-District-Profile-Report-Final-12-12-2024.xlsx.aspx?lang=en-US"
        ),
        filename="cupp-fy24.xlsx",
        catalog="cupp-district-profile-report",
        note="60 variables per district. Fiscal and tax years are mixed within a row.",
    ),
    "enrollment-fy24": Source(
        key="enrollment-fy24",
        url=(
            "https://education.ohio.gov/getattachment/Topics/Data/Frequently-Requested-Data/"
            "Enrollment-Data/oct_hdcnt_fy24.xls.aspx?lang=en-US"
        ),
        filename="oct_hdcnt_fy24.xls",
        catalog="cupp-district-profile-report",
        note="October headcount by individual grade. Legacy .xls; needs conversion.",
        legacy_xls=True,
    ),
}


class RetrievalError(RuntimeError):
    """A source could not be fetched or converted."""


def fetch(source: Source, cache: Path | None = None, refresh: bool = False) -> Path:
    """Download ``source`` into the cache and return the local path.

    Cached by filename. Pass ``refresh=True`` to re-download; otherwise an existing file is
    returned untouched, so repeated extraction runs make no network requests.
    """
    cache_dir = Path(cache) if cache else DEFAULT_CACHE
    cache_dir.mkdir(parents=True, exist_ok=True)
    dest = cache_dir / source.filename
    if dest.exists() and not refresh:
        return dest

    req = urllib.request.Request(source.url, headers={"User-Agent": USER_AGENT})
    try:
        with urllib.request.urlopen(req, timeout=180) as r, dest.open("wb") as out:
            shutil.copyfileobj(r, out)
    except Exception as exc:  # noqa: BLE001 - surface the cause, whatever urllib raised
        raise RetrievalError(f"could not fetch {source.key} from {source.url}: {exc}") from exc
    return dest


def convert_legacy_xls(path: Path) -> Path:
    """Convert an OLE2 ``.xls`` to ``.xlsx`` using headless LibreOffice.

    The department still publishes enrollment data in the pre-2007 format, which is not a zip
    and therefore unreadable by :mod:`.xlsx`. LibreOffice is the only converter available
    without adding a dependency, and it must be installed separately.

    Converting to ``.xlsx`` rather than straight to CSV is deliberate: the CSV filter exports
    only the active sheet, and the enrollment workbook keeps district data on the third of
    seven.
    """
    out_dir = path.parent / "converted"
    out_dir.mkdir(parents=True, exist_ok=True)
    converted = out_dir / (path.stem + ".xlsx")
    if converted.exists():
        return converted

    soffice = shutil.which("soffice") or shutil.which("libreoffice")
    if soffice is None:
        raise RetrievalError(
            f"{path.name} is a legacy .xls and needs LibreOffice to convert. "
            "Install it, or convert the file by hand and place the result at "
            f"{converted}."
        )
    result = subprocess.run(  # noqa: S603 - fixed binary, no shell
        [soffice, "--headless", "--convert-to", "xlsx", "--outdir", str(out_dir), str(path)],
        capture_output=True,
        text=True,
        timeout=300,
        check=False,
    )
    if not converted.exists():
        raise RetrievalError(
            f"LibreOffice did not produce {converted}: {result.stdout} {result.stderr}"
        )
    return converted


def ensure_readable(source: Source, cache: Path | None = None, refresh: bool = False) -> Path:
    """Fetch a source and, if it is legacy ``.xls``, convert it. Returns a readable path."""
    path = fetch(source, cache=cache, refresh=refresh)
    return convert_legacy_xls(path) if source.legacy_xls else path
