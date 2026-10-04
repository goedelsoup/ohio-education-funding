---
used-by:
  - ../corpus/intervention/academic-distress-commission.yml
---
# Ohio School Report Cards — District High-Level Ratings

**Source.** Ohio Department of Education and Workforce, Ohio School Report Cards, district-level
high-level download: `DISTRICT_HIGH_LEVEL_2425.xlsx` (the 2024-25 report card, last modified
14 September 2025) and `DISTRICT_HIGH_LEVEL_2526.xlsx` (the 2025-26 report card, released 15
September 2026 and last modified 25 September 2026), by the storage listing.
**Type.** Primary source — machine-readable per-district ratings.
**Location.** `reportcardstorage.education.ohio.gov/data-download-2025/` and
`.../data-download-2026/`. Requires the read-only query token recorded in
[`decisions/report-card-connector`](../decisions/report-card-connector.yml). The token grants
list permission, so `?restype=container&comp=list` with it enumerates each year's files.

**What it contains.** Sheet `DISTRICT_OVERVIEW`: District IRN, the **overall star rating** and
its rating points, and each component's star rating beside its underlying measure —
achievement, gap closing, early literacy, progress, graduation, and college, career, workforce
and military readiness. One row for each of the 607 rated traditional districts in each
edition. Only IRN, name and the overall rating are extracted.

**Why it matters here.** It is the only held download that carries the overall rating, and the
overall rating is the one R.C. 3302.10 reads: three consecutive years below two stars
establishes an academic distress commission, and one year at three stars begins a transition
out. Every other report card file here carries one component.

**It also dates the calculator.** The FY2026 and FY2027 calculators label their overall-rating
column `O1` with no year. It equals the 2024-25 edition's overall rating for all 607 rated
districts, so the performance supplement in both years pays on the 2024-25 card. [verified]
([`crates/project/tests/the_supplements_outside_the_formula.rs`](../../crates/project/tests/the_supplements_outside_the_formula.rs))

**Access constraints.** Freely available, no registration. XLSX.

**Caveats:**

- **The layout moves between editions.** The 2025-26 file inserts a `Performance Index` column
  after `Performance Index Percent`, and writes numbers where 2024-25 wrote text. The extract
  reads columns by header name. [verified]
- **Two editions are held, not three.** R.C. 3302.10(A)(1) needs three consecutive years, so
  the trigger is still not computable from these alone. A 2023-24 edition is not registered.
  [open]
- **Zero is not a rating.** The calculators write 0 stars for four districts the download does
  not rate at all: College Corner, Kelleys Island, Middle Bass and North Bass. [verified]

## Feeds connector

[`dew-report-card`](../../crates/connect/src/registry/dew.rs), source keys
`district-high-level-2425` and `district-high-level-2526`.

## Also read by

[`crates/dispersion/fixtures/report-card-overall-ratings.csv`](../../crates/dispersion/fixtures/report-card-overall-ratings.csv),
one column per edition.
