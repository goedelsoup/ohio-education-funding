# District Profile Report — the "Cupp Report"

**Source.** Ohio Department of Education and Workforce, District Profile Reports, published
annually. Named after Bob Cupp — the same legislator who later co-authored the
[Fair School Funding Plan](../corpus/funding-regime/fair-school-funding-plan.yml).
**Type.** Primary source — machine-readable per-district finance data.
**Location.** `education.ohio.gov`, Finance and Funding → School Payment Reports → District
Profile Reports. The FY2024 edition is `FY24-District-Profile-Report-Final-12-12-2024.xlsx`,
about 1 MB.

**What it contains.** Sixty variables for each of Ohio's **606 traditional school districts**,
on five worksheets — District Data, Similar District Data, Statewide Data, and a formatted
one-page-per-district report. Demographics, personnel, property valuation and tax data, local
tax effort, expenditure, and revenue by source, each with comparison columns against similar
districts and the state.

This is the single most useful source the corpus has found. It carries, per district and in
one file: enrolled ADM, economically disadvantaged share, classroom teachers' average salary,
assessed valuation per pupil, current operating millage, **effective Class 1 and Class 2
millage**, operating expenditure per pupil, and revenue split by state, local, federal, and
other. It is what turned the exemplar agencies from identifiers into series and what let the
corpus answer how many districts sit at the 20-mill floor.

A seventeen-column extract is committed at
[`crates/dispersion/fixtures/cupp-fy24-district-data.csv`](../../crates/dispersion/fixtures/cupp-fy24-district-data.csv)
and is the basis of that crate's integration tests: the ten columns the equity findings read,
and since #408 the report's seven **personnel** columns. `edfund-connect rebuild` writes it from
the cached workbook through `connect::fixtures::build_profile_extract`.

**What the personnel family is, and is not.** Columns O through U of `District Data`:
classroom teachers' average salary; the share of teachers with 0-4, 4-10 and 10+ years'
experience; `FTE Number of Administrators`; administrators' average salary; and the
pupil-administrator ratio, which is the department's own quotient of enrolled ADM by the FTE
count. **One of the seven is a count of staff, and it is administrators.** There is no teacher
FTE and no position category finer than that. The report's page defines the count as the sum of
twelve EMIS position codes — administrative assistant (101, the licensed code), deputy
superintendent (103), assistant principal (104), principal (108), superintendent (109),
supervisor/manager (110), treasurer (112), coordinator (113), education administration specialist
(114), director (115), community school administrator (116) and other officials/administrators
(199). That is the union of R.C. 3317.011's four administrator elements at (F)(1)-(3) and (G)(1),
so the count can be set against the two-other-administrators floor and against nothing else in
that section; the six other staffing floors meet no column here, and the category detail they
would need lives in EMIS staff reporting, which is not catalogued.

**Access constraints.** Freely available. XLSX rather than CSV; it parses with stdlib `zipfile`
plus `xml.etree` — an XLSX is a zip of XML — with no third-party library required. The
`District Data` worksheet is `xl/worksheets/sheet2.xml`.

**Caveats, all observed rather than assumed:**

- **Fiscal and tax years are mixed within a single row.** Enrolled ADM is FY2024, assessed
  valuation per pupil is FY2023, millage is TY2023, and OFCC adjusted valuation is FY2025. The
  column headers state this; a naive read that treats a row as one year is wrong.
- **Missing values appear as `#N/A` and `#DIV/0!`**, not as blanks. Southern Local (046441),
  Columbiana County reports no operating expenditure per pupil at all, so statewide
  expenditure statistics are computed over 605 districts, not 606.
- **Published rounding produces impossible-looking rows.** Five districts report an effective
  Class 1 rate marginally *above* their voted operating millage — up to 0.0004 mills, e.g.
  22.65 effective against 22.6499 voted. These are artifacts, not violations, and a validity
  check with too tight a tolerance will reject good data.
- **Traditional districts only.** No community schools, no STEM schools, and no JVSDs, which is
  why the corpus's JVSD exemplar cannot be populated from here.
- **The department revised the FY2024 edition a year after publishing it.** The cached and
  connected file is `FY24-District-Profile-Report-Final-12-12-2024.xlsx` (workbook modified
  2024-12-27). The report's page now serves
  `FY24-District-Profile-Report-Revised-12-18-2025.xlsx` (modified 2025-12-18), and the original
  URL still resolves. Compared cell by cell on 2026-09-22: the revision rewrites the teachers'
  salary and three experience-share columns for about 300-540 districts and the administrators'
  salary for 439, restates 25 district names with a doubled space, and moves nothing else beyond
  float noise past the fourth decimal. **`FTE Number of Administrators` is identical in both**,
  as are the ten columns the equity findings read, so the extract is taken from the connected
  edition and the personnel findings do not depend on which. A re-vendor to the revised edition
  would move the four salary and experience columns and nothing the corpus binds.

## Used by

- [`parameter/twenty-mill-floor`](../corpus/parameter/twenty-mill-floor.yml)
- [`metric/assessed-valuation-per-pupil`](../corpus/metric/assessed-valuation-per-pupil.yml)
- [`metric/effective-operating-millage`](../corpus/metric/effective-operating-millage.yml)
- [`metric/per-pupil-operating-expenditure`](../corpus/metric/per-pupil-operating-expenditure.yml)
- [`doctrine/equity`](../corpus/doctrine/equity.yml)
- [`education-agency/northern-local-perry`](../corpus/education-agency/northern-local-perry.yml)
- [`education-agency/upper-arlington-city`](../corpus/education-agency/upper-arlington-city.yml)
- [`education-agency/cleveland-municipal`](../corpus/education-agency/cleveland-municipal.yml)
- [`formula-component/fsfp-base-cost-calculation`](../corpus/formula-component/fsfp-base-cost-calculation.yml)
  — the administrator count against the section's funded administrators

## Feeds connector

[`dew-foundation`](../../crates/connect/sources/dew-foundation.md), and via `tax-abstract` for the
millage and valuation columns.
