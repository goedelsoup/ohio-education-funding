# Ohio School Report Cards — District Value-Added Details

**Source.** Ohio Department of Education and Workforce, Ohio School Report Cards, district-level
value-added download, file `2425_VA_DIST_DETAILS.xlsx`, generated September 11, 2025.
**Type.** Primary source — machine-readable per-district growth data.
**Location.** `reportcardstorage.education.ohio.gov/data-download-2025/`. Requires the read-only
query token recorded in [`decisions/report-card-connector`](../decisions/report-card-connector.yml).

**What it contains.** Four sheets. `OVERALL_VALUE_ADDED_OVERVIEW` is the usable one: District IRN,
Progress Component star rating, `Overall Composite`, and `Overall Effect Size`, one row for each
of the same 607 rated traditional districts. The three `*_GAINS` sheets break the same measure
out by grade and subject over one-, two-, and three-year windows and are not extracted. [open]

**Why it matters here.** The Performance Index measures where a district's students *are*; this
measures how far they *moved*. Given that the Index is 71.6% economically disadvantaged share,
that distinction carries most of what this corpus can say about outcomes.

The measures behave completely differently against the same spending variable:

| | vs Performance Index (level) | vs Progress effect size (growth) |
|---|---|---|
| economically disadvantaged share | −0.846 | −0.325 |
| spending per unweighted pupil, raw | −0.337 | +0.018 |
| the same, holding disadvantage constant | **−0.125** | **+0.146** |

The sign flips. [verified — see
[`crates/dispersion/tests/report_card_2425.rs`](../../crates/dispersion/tests/report_card_2425.rs)]

**Use the effect size, not the composite.** `Overall Composite` is a precision-scaled statistic —
a gain over its standard error — so it grows with the number of tested students and correlates
with enrollment at **+0.244**. `Overall Effect Size` is the standardised gain and correlates with
enrollment at +0.155. The two correlate with each other at +0.917, so the choice looks immaterial
and is not: ranking districts on the composite ranks them partly by size. [verified]

**Access constraints.** Freely available, no registration. XLSX.

**Caveats:**

- **Growth is not composition-free, only much less composition-bound.** The effect size still
  tracks the economically disadvantaged share at −0.325, about 10.6% of its variance, against
  71.5% for the Index. Treating Progress as a clean measure of school contribution overstates
  what it does. [verified]
- **It is centred by construction.** Mean effect size −0.003, median 0.000, standard deviation
  0.083, range −0.29 to +0.29. Ohio's value-added model is normed so the state average is zero,
  which means this measure cannot say whether Ohio as a whole is improving — only which districts
  moved more than others. Any across-year or against-state reading of it is wrong. [verified]
- **One year of it is held.** The overview sheet is the 2024-25 composite. The multi-year gain
  sheets are in the file and unextracted, so the corpus cannot yet ask whether a district's
  growth is stable the way it has asked of the Index. [open]
- **Same 607-district rated population** as the achievement and spending files, matching on IRN
  with no losses. [verified]

## Used by

- [`metric/progress-value-added`](../corpus/metric/progress-value-added.yml)
- [`metric/performance-index`](../corpus/metric/performance-index.yml)
- [`metric/expenditure-per-equivalent-pupil`](../corpus/metric/expenditure-per-equivalent-pupil.yml)

## Feeds connector

[`dew-report-card`](../../crates/connect/src/registry.rs), source key `va-district-details-2425`.
