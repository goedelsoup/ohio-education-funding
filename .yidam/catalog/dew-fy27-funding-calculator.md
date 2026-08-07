# FY27 TRAD State Foundation Funding Calculator

**Source.** Ohio Department of Education and Workforce, Office of Budget and School Funding.
`FY27-TRAD-State-Foundation-Funding-Calculator_12-16-2025_lock-1.xlsx`, dated 16 December 2025.
**Type.** Primary source — the department's own working model, not a report about it.
**Location.** `education.ohio.gov`, Finance and Funding → School Payment Reports → State Funding
for Schools → Traditional School Districts.

**What it contains.** Thirty-three worksheets comprising the complete Fair School Funding Plan
computation for FY2027, the terminal year of the six-year phase-in. Statewide factor values;
per-district tables for base cost, local capacity, targeted assistance, DPIA, English learner,
gifted, career-technical, special education, transportation, and preschool special education;
ADM and valuation data; and forward projections of tax returns, median income, and federal AGI
for TY2027.

This is the most valuable single artefact the corpus has retrieved, for three reasons.

**It gives the verified FY2022 reference-year inputs.** The corpus had been carrying $67,654
for the FY2022 classroom teacher salary on secondary reporting. The department's own figure is
**$68,022.22** [s5d]. Every other cost input is here too — the insurance term moved from
$14,265.53 to $17,152.68, a 20% rise against the teacher salary's 8.5%.

**It carries the guarantee per district.** Column [I] of `Summary_SFPR`, Temporary Transitional
Aid Guarantee, is what let the corpus finally quantify the guarantee population — the largest
outstanding caveat on its scenario result.

**It is a verification target.** The `Base_Cost` sheet publishes funded teacher counts, teacher
base cost, aggregate base cost, and base cost per pupil for all 609 districts. The corpus's
[`foundation`](../../crates/foundation/src/lib.rs) crate reproduces the department's teacher
base cost to within a cent for every one of them, against a factor set it was never written
against. A ten-column extract is committed at
[`crates/foundation/fixtures/fy27-department-model.csv`](../../crates/foundation/fixtures/fy27-department-model.csv).

**Access constraints.** Freely available, 5 MB XLSX. Parses with stdlib `zipfile` plus
`xml.etree`. Cached formula results are present in the `<v>` elements, so values read without
needing a spreadsheet engine.

**Caveats.**

- **`Summary_SFPR` includes a "State of Ohio" aggregate row.** Treating the sheet as 612
  districts double-counts every total. There are 611 districts plus one statewide row, and the
  guarantee sums to $878,974,300 once, not twice.
- **Display sheets and data sheets are different things.** `Base Cost`, `Local Capacity`, and
  `Summary SFPR` (with spaces) are single-district display views driven by a selector. The
  per-district tables are the underscore variants: `Base_Cost`, `Local_Capacity`,
  `Summary_SFPR`.
- **It is a projection, not an actual.** FY2027 has not happened. Enrollment, valuation, and
  income are projected — the workbook has explicit projection sheets for TY2027 tax returns,
  median income, and FAGI. Figures from it describe what the department expects the formula to
  produce, not what it produced.
- Three statutory values are not exposed: the substitute daily rate and the superintendent and
  treasurer salary bands. The corpus assumes them unchanged from FY2022, an assumption the
  cent-level agreement on teacher base cost confirms for the substitute rate.

## Used by

- [`scenario/fsfp-input-year-refresh`](../corpus/scenario/fsfp-input-year-refresh.yml)
- [`parameter/base-cost-per-pupil`](../corpus/parameter/base-cost-per-pupil.yml)
- [`formula-component/fsfp-base-cost-calculation`](../corpus/formula-component/fsfp-base-cost-calculation.yml)
- [`funding-regime/fair-school-funding-plan`](../corpus/funding-regime/fair-school-funding-plan.yml)
- [`fiscal-period/fy2027`](../corpus/fiscal-period/fy2027.yml)

## Feeds connector

[`dew-foundation`](../../crates/dew-foundation/README.md)
