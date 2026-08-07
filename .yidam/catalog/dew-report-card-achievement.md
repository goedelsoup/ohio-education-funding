# Ohio School Report Cards — District Achievement Download

**Source.** Ohio Department of Education and Workforce, Ohio School Report Cards, district-level
Achievement download. The 2024–2025 edition is `24-25_Achievement_District.xlsx`, generated
September 11, 2025.
**Type.** Primary source — machine-readable per-district outcome data.
**Location.** `reportcard.education.ohio.gov`, Download Data → Achievement, district level.

**What it contains.** One row per district. The field this corpus needs is
`Performance Index Score 2024-2025` on the `Performance_Index` sheet — a continuous score built
from the distribution of tested students across Ohio's achievement levels, with higher levels
weighted more. The sheet also carries the indicators-met and achievement-component ratings.

**Why it matters here.** This is the corpus's only route to an *outcome* variable. Every metric
in [`corpus/metric/`](../corpus/metric/) before it measured an input — expenditure, valuation,
millage, state share. The Performance Index is the first measure of what the system produced
rather than what it spent, and it is what makes the outcome formulation of
[adequacy](../corpus/doctrine/adequacy.yml) testable rather than merely contested.

**Access constraints.** Freely available, no registration. XLSX.

**Caveats, from the file's own structure and DEW's published methodology:**

- **Coverage is the rated population, not the funded population.** The 2024–2025 file assigns a
  Performance Index to **607 traditional districts**. The corpus's other district cross-section,
  the [Cupp Report](cupp-district-profile-report.md), carries **606**, and the FY2027 department
  model carries **609**. Three counts, three populations, one year apart. They are not
  interchangeable and must not be pooled without a crosswalk. [verified]
- **Continuous, unlike the star rating.** The overall star rating collapses many districts into
  one category; the Index does not. Any analysis that needs to rank districts must use the
  Index, and any analysis quoting the star rating is answering a coarser question.
- **It is one construct.** The Index summarizes state-test results for tested students. It is
  not a measure of school quality, and it is not the Progress (value-added) component, which is
  designed to capture growth independent of starting achievement and is a different file.
- **Keyed by District IRN**, which is the join key the rest of this corpus already uses.

**Not yet extracted.** No Performance Index values are committed to `crates/` as of this entry.
Joining the FY2025 Index onto
[`crates/dispersion/fixtures/cupp-fy24-district-data.csv`](../../crates/dispersion/fixtures/cupp-fy24-district-data.csv)
by IRN is the single cheapest expansion available to the corpus — it would make the
spending–outcome and poverty–outcome relationships computable from committed data rather than
quoted from a secondary source. [open]

## Used by

- [`metric/performance-index`](../corpus/metric/performance-index.yml)
- [`catalog/ocg-white-paper-013`](ocg-white-paper-013.md)

## Feeds connector

None yet. A `dew-report-card` connector does not exist; the nine approved in
[`decisions/proposals`](../decisions/proposals.yml) are all finance-side. [open]
