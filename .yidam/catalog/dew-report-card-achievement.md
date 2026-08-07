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

**Extracted.** All 607 Performance Index values, plus the 2023-24 and 2022-23 scores in the same
row, are committed at
[`crates/dispersion/fixtures/report-card-2425-district-data.csv`](../../crates/dispersion/fixtures/report-card-2425-district-data.csv).
The join to the corpus's FY2024 cross-section is clean: 606 of the 607 match the Cupp Report on
IRN, and all 607 match the report card's spending and Expanded List files.

The first thing it showed: the Index tracks the economically disadvantaged share at **−0.846**,
71.6% of its cross-district variance. Any district-level correlate of this measure is partly that
variable in disguise. [verified]

**The two prior years are committed and unread.** Nothing in the corpus yet asks whether a
district's Index is stable enough across three years for a single-year cross-section to stand in
for it — a question this file can answer without any further retrieval. [open]

## Used by

- [`metric/performance-index`](../corpus/metric/performance-index.yml)
- [`catalog/ocg-white-paper-013`](ocg-white-paper-013.md)

## Feeds connector

[`dew-report-card`](../../crates/connect/src/registry.rs), source key
`achievement-district-2425` — the tenth connector, approved in
[`decisions/report-card-connector`](../decisions/report-card-connector.yml) and the first
retrieving an outcome rather than a dollar.
