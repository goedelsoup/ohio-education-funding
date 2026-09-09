# Ohio School Report Cards — District Achievement Download

**Source.** Ohio Department of Education and Workforce, Ohio School Report Cards, district-level
Achievement download. The 2024–2025 edition is `24-25_Achievement_District.xlsx`, generated
September 11, 2025.
**Type.** Primary source — machine-readable per-district outcome data.
**Location.** `reportcard.education.ohio.gov`, Download Data → Achievement, district level.

**What it contains.** One row per district on the `Performance_Index` sheet: the Performance
Index for 2024-25 and the two years before it, the `Achievement Component Star Rating`, the
Index expressed as a percentage, the statewide maximum that percentage is taken against, and —
the columns that make this file more than a score — **the district's tested students distributed
across all seven achievement levels**, from not tested through advanced plus.

**Those seven columns are why the weights are knowable.** The index is a weighted sum over that
distribution and DEW publishes the weights nowhere this corpus holds. Printing the distribution
beside the result, 607 times, is an overdetermined system in seven unknowns, and it solves: 100
for a district whose tested students are all proficient, and tenths from there, with an untested
student counting zero. See
[`crates/dispersion/tests/the_weights_behind_the_index.rs`](../../crates/dispersion/tests/the_weights_behind_the_index.rs).
[verified]

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
- **And the rating is relative, which is the stronger objection.** `Maximum District Performance
  Index Score` repeats one value — 109.8 — on every row, because R.C. 3302.03(D)(1)(c) defines
  it as the average of the highest two per cent of district scores *for the year the report card
  is issued*. It is refitted annually: 108.8 in 2022-23, 109.8 in 2024-25. The achievement rating
  is awarded on the Index over that, so it is a share of the top of the state and not a level,
  and 121 districts rose on the Index across those three years while falling on the share.
  Summing or averaging the maximum column measures nothing. [verified]
- **The percentage is computed before the score is rounded.** Dividing the published Index by the
  published maximum disagrees with the published percentage by a tenth on 131 of 607 districts.
  That is double rounding, not disagreement — a single unrounded index is consistent with both,
  every time. Anything reconstructed from the one-decimal columns inherits a tenth of slack, and
  the seven level shares are each rounded on their own, so they sum to between 99.8 and 100.2
  rather than to 100. [verified]
- **It is one construct.** The Index summarizes state-test results for tested students. It is
  not a measure of school quality, and it is not the Progress (value-added) component, which is
  designed to capture growth independent of starting achievement and is a different file.
- **Keyed by District IRN**, which is the join key the rest of this corpus already uses.

**Extracted.** All 607 Performance Index values, the 2023-24 and 2022-23 scores in the same row,
the achievement star rating, the Index percentage and its statewide maximum, and the seven
achievement-level shares are committed at
[`crates/dispersion/fixtures/report-card-2425-district-data.csv`](../../crates/dispersion/fixtures/report-card-2425-district-data.csv).
The join to the corpus's FY2024 cross-section is clean: 606 of the 607 match the Cupp Report on
IRN, and all 607 match the report card's spending and Expanded List files.

The first thing it showed: the Index tracks the economically disadvantaged share at **−0.846**,
71.6% of its cross-district variance. Any district-level correlate of this measure is partly that
variable in disguise. [verified]

**The two prior years are read.** Adjacent years correlate at +0.988 and +0.989 and 2022-23
against 2024-25 at +0.981, with a median within-district three-year range of 2.1 points — so a
single-year cross-section does stand in for a district's Index, and the measure cannot detect
anything that changes annually. [verified] All three sit inside one statutory regime: R.C.
3302.03(D) governs the 2021-22 school year and every year after it, so the stability is not a
redesign preserving ranks. [verified]

## Used by

- [`metric/performance-index`](../corpus/metric/performance-index.yml)
- [`accountability-regime/ohio-report-card`](../corpus/accountability-regime/ohio-report-card.yml)
- [`catalog/ocg-white-paper-013`](ocg-white-paper-013.md)

## Feeds connector

[`dew-report-card`](../../crates/connect/src/registry/dew.rs), source key
`achievement-district-2425` — the tenth connector, approved in
[`decisions/report-card-connector`](../decisions/report-card-connector.yml) and the first
retrieving an outcome rather than a dollar.
