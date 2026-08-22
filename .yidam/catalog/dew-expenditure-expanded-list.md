# FY2025 Expenditure Expanded List

**Source.** Ohio Department of Education and Workforce, FY2025 Expenditure Expanded List, file
`FY25 Expanded List.xlsx`, published in the Financial Expenditures category of the Ohio School
Report Card data downloads.
**Type.** Primary source — machine-readable per-agency expenditure data.
**Location.** `reportcardstorage.education.ohio.gov/data-download-2025/`, enumerated by the report
card's public download API. Requires the read-only query token recorded in
[`decisions/report-card-connector`](../decisions/report-card-connector.yml); without it the host
answers 404 rather than 403, so an omitted token looks like a moved file.

**What it contains.** Four sheets. `Overall Expenditure Data` carries the dollar totals by
function. The two that matter here are `Expenditure per Equivalent Pup` and
`Expenditure per Pupil`, which are **the same file twice**: identical layout, identical
`Operating Expenditures` column, and a column 3 that holds `Weighted ADM` on the first and
`Unweighted ADM` on the second.

**This is the most useful single fact in the file.** One numerator, two published denominators,
per IRN, in one workbook. Every other Ohio spending comparison has to choose a denominator and
defend it; this file lets both be computed and the difference measured. It is what converted the
corpus's objection to
[OCG White Paper 013](ocg-white-paper-013.md) from an argument into
[a test](../../crates/dispersion/tests/report_card_2425.rs): correlating the Performance Index
against the same expenditures divided by the two counts gives −0.015 and −0.337.

**Access constraints.** Freely available, no registration. XLSX.

**Caveats, all observed rather than assumed:**

- **The two ADM columns are published at different precisions.** `Unweighted ADM` carries four
  decimals; `Weighted ADM` is rounded to whole pupils. Recomputing a per-equivalent-pupil figure
  therefore reproduces the published one only within the rounding, and the residual scales
  inversely with size — Put-in-Bay Local at 77 weighted pupils lands $86 below its published
  $46,716, while Akron City at 29,162 lands within a cent. This is not an error in either
  figure, but it means small-district per-weighted-pupil values carry quantisation noise the
  headcount values do not. [verified]
- **Coverage is wider than the report card's district files.** 607 public districts, 320
  community schools, 49 JVSDs, 19 eschools, 8 STEM schools, and a handful of rows with no org
  type. Filter on `Org Type == "Public District"` before joining; an unfiltered join drops
  silently to the intersection and looks like it worked. [verified]
- **The weighting is DEW's, and its formula is not in this file.** Weighted ADM adjusts for
  economically disadvantaged, English-learner, and disability enrollment. The ratio of the two
  columns runs from 1.05 to 1.62 with a median of 1.26, correlates with the economically
  disadvantaged share at **+0.800**, and with the Performance Index at **−0.745**. It is very
  nearly a poverty index. The corpus does not hold the weight schedule that produces it. [open]
- **Operating expenditures, not all funds.** The relationship to the report card's own
  all-funds spending file has not been established. [open]

## Used by

- [`metric/expenditure-per-equivalent-pupil`](../corpus/metric/expenditure-per-equivalent-pupil.yml)
- [`metric/per-pupil-operating-expenditure`](../corpus/metric/per-pupil-operating-expenditure.yml)
- [`metric/performance-index`](../corpus/metric/performance-index.yml)

## Feeds connector

[`dew-report-card`](../../crates/connect/src/registry/dew.rs), source key `expanded-list-fy25`.
Extracted into
[`crates/dispersion/fixtures/report-card-2425-district-data.csv`](../../crates/dispersion/fixtures/report-card-2425-district-data.csv).
