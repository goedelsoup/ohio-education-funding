# October headcount by grade

**Source.** Ohio Department of Education and Workforce, Frequently Requested Data → Enrollment
Data, October headcount. The FY2024 edition is `oct_hdcnt_fy24.xls`, about 2.1 MB.
**Type.** Primary source — per-district enrollment by individual grade.

**What it contains.** A single-day headcount for every district, broken out by **individual
grade** — kindergarten through twelve, separately — rather than by the grade bands the funding
formula uses. Also disaggregated by race and gender on companion sheets.

**Why it matters here.** The Fair School Funding Plan's base cost prices teachers at four
different pupil-teacher ratios: kindergarten, grades 1–3, grades 4–8, and grades 9–12, plus a
separate career-technical ratio. The department publishes the *banded* ADM in its funding
calculator, but only this file publishes the grades that make up each band. It is what allows
an independent check that the bands account for the reported headcount rather than assuming it.

A five-column extract is committed at
[`crates/foundation/fixtures/fy24-district-grade-bands.csv`](../../crates/foundation/fixtures/fy24-district-grade-bands.csv).

**Access constraints.** Free. **Still published in the pre-2007 OLE2 `.xls` format**, which is
a compound document, not a zip, and shares nothing with XLSX beyond the extension's family
resemblance. [`crates/spreadsheet`](../../crates/spreadsheet/) does not read it; conversion is
delegated to headless LibreOffice, and this is consequently the **one source in the registry
whose extraction is not reproducible from a checkout alone**. Reading it natively means an OLE2
sector walker and a BIFF8 record parser; that is the honest completion of this connector and it
is not done.

**Caveats:**

- **Headcount is not ADM.** This is a single-day count; average daily membership is
  time-weighted across the year and is the quantity the formula actually funds. The two differ
  by a percent or two per district and by more where enrollment moved mid-year.
- **Small counts are suppressed as `<10`, not withheld as blank.** Treating that as zero
  understates any aggregate over small districts — which are exactly the districts a
  school-funding question is usually about. See `crates/connect/src/conventions.rs`.
- **District data is on the third of seven sheets**, which is why conversion targets `.xlsx`
  rather than CSV: LibreOffice's CSV filter exports only the active sheet.

## Used by

- [`formula-component/`](../corpus/formula-component/) — the grade-band pupil-teacher ratios
- [`metric/`](../corpus/metric/)

## Feeds connector

[`dew-foundation`](../../crates/connect/sources/dew-foundation.md).
