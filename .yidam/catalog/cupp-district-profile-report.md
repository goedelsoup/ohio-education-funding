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

A ten-column extract is committed at
[`crates/dispersion/fixtures/cupp-fy24-district-data.csv`](../../crates/dispersion/fixtures/cupp-fy24-district-data.csv)
and is the basis of that crate's integration tests.

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

## Used by

- [`parameter/twenty-mill-floor`](../corpus/parameter/twenty-mill-floor.yml)
- [`metric/assessed-valuation-per-pupil`](../corpus/metric/assessed-valuation-per-pupil.yml)
- [`metric/effective-operating-millage`](../corpus/metric/effective-operating-millage.yml)
- [`metric/per-pupil-operating-expenditure`](../corpus/metric/per-pupil-operating-expenditure.yml)
- [`doctrine/equity`](../corpus/doctrine/equity.yml)
- [`education-agency/northern-local-perry`](../corpus/education-agency/northern-local-perry.yml)
- [`education-agency/upper-arlington-city`](../corpus/education-agency/upper-arlington-city.yml)
- [`education-agency/cleveland-municipal`](../corpus/education-agency/cleveland-municipal.yml)

## Feeds connector

[`dew-foundation`](../../crates/dew-foundation/README.md), and via `tax-abstract` for the
millage and valuation columns.
