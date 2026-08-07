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
resemblance. [`crates/spreadsheet`](../../crates/spreadsheet/) reads it natively — an OLE2
sector walker and a BIFF8 record parser — so extraction from this source is reproducible from a
checkout with no external converter. It was not, until the reader was written: the fixture was
produced through headless LibreOffice, and that pipeline summed a withheld `<10` as zero.

**Caveats:**

- **Headcount is not ADM.** This is a single-day count; average daily membership is
  time-weighted across the year and is the quantity the formula actually funds. The two differ
  by a percent or two per district and by more where enrollment moved mid-year.
- **Small counts are suppressed as `<10`, not withheld as blank.** Treating that as zero
  understates any aggregate over small districts — which are exactly the districts a
  school-funding question is usually about. See `crates/connect/src/conventions.rs`.

  **Five districts in this file have a suppressed grade, and they are the five smallest in
  Ohio**: Kelleys Island, Put-in-Bay, College Corner, Vanlue, Bloomfield-Mespo. The first three
  are outside the committed extract already. For the other two the grade band containing the
  withheld grade is left **blank**, because a band summed over a `<10` is not a smaller band —
  it is a band whose total is unknown. The LibreOffice-derived fixture summed them as zero and
  recorded Vanlue's grades 9-12 as 56 where the true figure is between 57 and 65. [verified]
- **District data is on the third of seven sheets**, which is why conversion targets `.xlsx`
  rather than CSV: LibreOffice's CSV filter exports only the active sheet.

## Used by

- [`formula-component/`](../corpus/formula-component/) — the grade-band pupil-teacher ratios
- [`metric/`](../corpus/metric/)

## Feeds connector

[`dew-foundation`](../../crates/connect/sources/dew-foundation.md).
