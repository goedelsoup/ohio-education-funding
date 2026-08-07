# Ohio School Report Cards — District Details

**Source.** Ohio Department of Education and Workforce, Ohio School Report Cards, file
`2025_District_Details.xlsx`, sheet `District_Details`.
**Type.** Primary source — machine-readable per-district subgroup data.
**Location.** `reportcardstorage.education.ohio.gov/data-download-2025/`. Requires the read-only
query token recorded in [`decisions/report-card-connector`](../decisions/report-card-connector.yml).

**What it contains.** Long form — one row per district per student group, 5,820 rows over the same
607 rated traditional districts. Per row: `Enrollment`, `Enrollment Percent`, `Attendance rate`,
`Mobility Rate`, `Chronic Absenteeism Rate`. Eleven groups, of which `All Students`,
`Economic Disadvantage`, `English Learner`, and `Students with Disabilities` are the ones this
corpus reads.

**Why it matters here.** These are the need covariates, published for the same school year as the
outcomes, by the same publisher, on the same IRN key. They are what made a need-adjusted model
possible — the step [OCG White Paper 013](ocg-white-paper-013.md) named as its own priority and
could not take.

**THE ECONOMIC-DISADVANTAGE SHARE HERE IS NOT THE CUPP REPORT'S.** This is the most important
caveat in the file and it is not stated on the file.

|  | this file (2024-25) | [Cupp Report](cupp-district-profile-report.md) (FY2024) |
|---|---|---|
| median share | 49.5% | 47.0% |
| districts at exactly 100% | **87** | 37 |
| districts at 95% or more | 197 | — |
| correlation with the Performance Index | **−0.734** | **−0.846** |

The two correlate with each other at only +0.823, and individual districts disagree wildly — one
reports 94.4% here against 5.8% there. This is not a year of drift. This measure is **top-coded by
community eligibility**: where a district qualifies for universal free meals, every enrolled
student is counted economically disadvantaged, so the variable saturates at 100% and stops
discriminating among the poorest districts. Akron City reports an `Economic Disadvantage`
enrollment exactly equal to its `All Students` enrollment. [verified]

The censoring shows up exactly where censoring should: the saturated measure gives the *weaker*
association with the outcome. This corpus uses the Cupp measure for its headline poverty findings
and commits both. [verified — see
[`crates/dispersion/tests/report_card_2425.rs`](../../crates/dispersion/tests/report_card_2425.rs)]

**Access constraints.** Freely available, no registration. XLSX.

**Caveats:**

- **`NC` and `<10` are a matched pair.** Where a group has fewer than ten students the enrollment
  cell reads `<10` and the percent reads `NC` — 216 districts for English Learner. These are
  suppressed small counts, not absent data and not zero. Any share imputed at zero for them is a
  stated assumption; the corpus makes that assumption for English-learner share, notes that ten
  students in a district of ~1,500 is under 0.7%, and re-runs the model on complete cases as a
  check. [verified]
- **Attendance, mobility, and chronic absenteeism are outcomes, not covariates.** They sit in the
  same rows and are tempting. Putting them on the right-hand side of a spending-outcome model
  controls away a mediator and biases the coefficient of interest toward zero. This corpus does
  not use them as controls. [inference]
- **Long form.** Keyed by (IRN, student group). A join on IRN alone silently multiplies rows.
- **Group coverage varies** — 607 districts report Students with Disabilities, 606 Economic
  Disadvantage, 303 a numeric English Learner share, 119 Migrant.

## Used by

- [`metric/performance-index`](../corpus/metric/performance-index.yml)
- [`metric/progress-value-added`](../corpus/metric/progress-value-added.yml)
- [`metric/expenditure-per-equivalent-pupil`](../corpus/metric/expenditure-per-equivalent-pupil.yml)

## Feeds connector

[`dew-report-card`](../../crates/connect/src/registry.rs), source key `district-details-2425`.
