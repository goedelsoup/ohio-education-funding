# FY27 Community/STEM School State Foundation Funding Simulator

**Source.** Ohio Department of Education and Workforce, Office of Budget and School Funding.
`FY27-CS-State-Foundation-Funding-Calculator-12-24-2025.xlsx`, last modified 24 December 2025 by
its own `docProps` — the filename's date and the file's agree.
**Type.** Primary source — the department's own working model, not a report about it.
**Location.** `education.ohio.gov`, Finance and Funding → School Payment Reports → State Funding
for Schools → Community School Funding. One directory across from
[`dew-fy27-funding-calculator`](dew-fy27-funding-calculator.md), which is the same office's model
of the same formula for the other population.

**What it contains.** Twenty-eight worksheets computing FY2027 state foundation funding for
**355 community and STEM schools**: base cost, special education, DPIA, English learner and
career-technical by category, core foundation funding, transportation, the equity supplement, the
formula transition supplement, the base funding supplement, and facilities. Plus enrolled ADM,
the FY2020 and FY2021 funding bases the transition supplement is computed against, and a school
list.

This is the only **per-school** formula source the corpus has retrieved, and the only one that
carries the line below.

**The equity supplement, which is why this was retrieved.** R.C. 3317.022, as H.B. 96 codified
it, pays a flat per-pupil amount to site-based community schools. The workbook states the rate in
a header cell rather than leaving it to be inferred — `Detail SFPR` row 2, *Equity Suplement per
pupil* (the department's spelling), against **`400`** — and every recipient's payment is that
rate times its enrolled ADM, to the cent. Statewide it is **$34,650,906.26** over **86,627.3**
enrolled ADM.

**It is not paid to every school in the file, and the enacted rule is not the proposed one.** Of
the 355 schools, **324 site-based schools receive it and 31 do not**: 23 e-schools and 8 STEM
schools, paid exactly zero. That is the enacted rule — item 10 of the act's own list pays
"community schools that are not internet- or computer-based", and the
[greenbook](lsc-hb96-analysis.md) says "site-based community schools". **The
[redbook](lsc-dew-redbook.md) says the opposite**, because it describes the *introduced* budget:
"[t]he executive proposal codifies the equity supplement into permanent law and expands
eligibility to STEM schools", at $58.0 million a year. That expansion did not survive into the
act, and this workbook is the department's own confirmation that it did not.

The split is not visible in the money. It is carried by the `E-School Designation` column, which
answers two questions in one — `N`, `Y` and `STEM`, where `Y` and `STEM` are both "not site-based"
and are not each other. A seventeen-column extract is committed at
[`crates/dispersion/fixtures/fy27-community-school-funding.csv`](../../crates/dispersion/fixtures/fy27-community-school-funding.csv),
built by
[`connect::fixtures::community_schools`](../../crates/connect/src/fixtures/community_schools.rs),
which refuses the file if a site-based school's payment is not rate times ADM or if an e-school or
STEM school is paid anything at all.

**Access constraints.** Freely available, 1,583,358 bytes XLSX, no credentials. Cached formula
results are present, so values read without a spreadsheet engine. The workbook carries a
structure-lock password; it does not impede reading.

**Caveats.**

- **The space-versus-underscore rule from the district calculator does not hold in this file.**
  There, an underscore names the per-district data sheet and a space names a single-district
  display view. Here the per-school detail sheet is **`Detail SFPR`, with a space**, and the
  display view beside it is **`Detailed SFPR `** with a trailing one. The summary pair does follow
  the rule — `Summary_SFPR` is the 356-row data sheet, `Summary SFPR` a 37-row printed report.
  Applying the district rule to the detail sheet gets the printout.

- **`Summary_SFPR (2)` is a stale sheet whose supplement column is not the supplement.** It is a
  hidden leftover of the FY2026 model — its phase-in cell reads `0.8333` where this year's reads
  `1` — and it carries 358 rows against the live sheet's 356. Its column headed
  **`H. Equity Supplement` holds total state support**: on 352 of the 353 schools the two sheets
  share it equals `Summary_SFPR`'s `L. Total State Supports`, and on **none** of them does it
  equal the equity supplement. A reader locating the column by its heading would report $1.5
  billion where $34.7 million belongs, row by plausible row. This is the shape
  `dew-fy27-funding-calculator` records for its own headers: **a column header in one of these
  workbooks is not evidence of what the column holds.**

- **`Summary_SFPR` includes a "State of Ohio" aggregate row**, IRN `050765`, as the district
  workbook does. Treating the sheet as 356 schools double-counts every total. The extract holds
  the 355 and uses the aggregate to check them: the schools it keeps sum to the department's own
  statewide supplement to the cent.

- **Three other rates sit in the same header band, and only one of them is this supplement's.**
  `Detail SFPR` row 1–2 also carry the base funding supplement at `$40` a pupil, brick-and-mortar
  facilities at `$1,000`, e-school facilities at `$25`, and a facilities proration of `0.9788782`.
  The base funding supplement is $40 a pupil for **all 355 schools including the e-schools and
  STEM schools**, and facilities pays STEM schools at the brick-and-mortar rate. So the workbook
  does discriminate on this one line specifically, and a reading that took "STEM schools are
  funded differently" as a general property of the file would be wrong in both directions.

- **It is a projection, not an actual.** FY2027 has not happened, and the file's own word for
  itself is *simulator*. Enrolled ADM is projected. Figures from it describe what the department
  expects the formula to produce.

- **No local share exists to be missing.** Community and STEM schools have no taxing authority,
  so the state share is 100% and the valuation, income and capacity columns the district model
  turns on are absent by construction rather than unpublished.

- **Two of the schedule's three years have a model, and the third never will.** The rate
  schedule the act sets is $650 in FY2025, $500 in FY2026 and $400 in FY2027. This file verifies
  the last; the FY2025 sibling, archived whole, verifies the first — see
  [`dew-fy25-community-school-calculator`](dew-fy25-community-school-calculator.md). **FY2026 has
  no model and is not pending one.** The department publishes one community school calculator at
  a time and replaces it in place, and the FY2026 edition was replaced before any crawler took a
  copy: the Internet Archive holds exactly **two** of these workbooks across the whole
  `State-Funding-For-Schools` tree, FY2019 and FY2025, and the archived captures of the community
  school page itself link no calculator in any month of 2025. This workbook's own hidden leftover
  sheet from the prior year states no $500 either, and the department's FY2026 actuals are school
  totals with no line breakout. So FY2026's $500 rests on the enacted act and the greenbook,
  permanently. Nothing here implies a retrieval someone could go and make.

## Used by

- [`formula-component/fsfp-community-school-equity-supplement`](../corpus/formula-component/fsfp-community-school-equity-supplement.yml)
- [`legislation/hb-96-2025`](../corpus/legislation/hb-96-2025.yml)

## Feeds connector

[`dew-foundation`](../../crates/connect/sources/dew-foundation.md)
