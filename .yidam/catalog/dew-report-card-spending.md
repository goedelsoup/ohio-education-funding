# Ohio School Report Cards — District Spending Per Pupil

**Source.** Ohio Department of Education and Workforce, Ohio School Report Cards, district-level
spending download. The 2024–2025 edition is `2425_DISTRICT_SPEND_PER_PUPIL.xlsx`, generated
November 25, 2025.
**Type.** Primary source — machine-readable per-district expenditure data.
**Location.** `reportcard.education.ohio.gov`, Download Data → Spending, district level.

**What it contains.** One row per district on the `DISTRICT_SPENDING_PER_PUPIL` sheet, keyed by
District IRN: `Expenditures per Equivalent Pupil` and its two components, `– Federal Funds` and
`– State and Local Funds`. Figures come from DEW's Expenditure Flow Model, which organizes
districts' end-of-year financial records into comparable categories.

**The denominator is the whole story.** This file reports expenditure per *equivalent* pupil.
The denominator is an average-daily-membership count weighted upward by DEW formulas for
economically disadvantaged, English-learner, and disability enrollment — so a high-need district
divides its spending by a larger number than its headcount and reports a lower figure. This is
not the per-pupil expenditure most Ohio arguments quote, and it is not
[`metric/per-pupil-operating-expenditure`](../corpus/metric/per-pupil-operating-expenditure.yml),
which uses a headcount denominator. See
[`metric/expenditure-per-equivalent-pupil`](../corpus/metric/expenditure-per-equivalent-pupil.yml)
for what the difference does to a correlation.

**Access constraints.** Freely available, no registration. XLSX.

**Caveats:**

- **Not comparable to headcount per-pupil figures.** For 2024–2025 this file reports a median of
  $12,856 and a mean of $13,224 across 607 districts. Within the Expanded List, where the same
  expenditures are divided both ways, the headcount median is $16,289 — the equivalent-pupil
  figure is **21% lower**, and that gap is the weighting alone, with no definitional difference
  to confound it. [verified]
- **The all-funds total is the sum of the two components**, so federal and state-and-local are
  not independent predictors of anything and must not be entered in one model as though they
  were. [verified — the components sum to the total in every published quintile row]
- **Whether residual federal relief is included is not stated on the file.** The corpus records
  that including or excluding COVID relief moved statewide per-pupil expenditure by $821 in
  FY2022 — larger than most policy changes it models. The 2024–2025 federal mean of $675 per
  equivalent pupil is small enough to be consistent with Title I and IDEA alone, so the relief
  tail is probably exhausted here, but the file does not say so and any use of the federal
  component should. [open]

**Companion file.** The [FY2025 Expenditure Expanded List](dew-expenditure-expanded-list.md)
carries `Unweighted ADM` and `Weighted ADM` per IRN on separate sheets over the same operating
expenditure total. It is the file that turned the denominator problem above from an argument into
a measurement: the Performance Index correlates with those expenditures at −0.015 per weighted
pupil and **−0.337** per unweighted pupil. [verified] Note that its published filename carries
spaces (`FY25 Expanded List.xlsx`), not the underscores cited in White Paper 013.

## Used by

- [`metric/expenditure-per-equivalent-pupil`](../corpus/metric/expenditure-per-equivalent-pupil.yml)
- [`catalog/ocg-white-paper-013`](ocg-white-paper-013.md)

## Feeds connector

[`dew-report-card`](../../crates/connect/src/registry/dew.rs), source key `spend-per-pupil-2425`.
