# County Student Distribution — gross casino revenue county student fund

**Source.** Ohio Department of Taxation, Revenue Accounting. Sixteen workbooks: `2015 08 Casino
Student Dist. by County by SD Web.xlsx` through `2024 01 County Student Distribution Report
Web.xlsx`, plus `FY 2016-FY2017 SD Distributions.xlsx`, which carries four distributions on four
sheets.
**Type.** Primary source — the department's own record of money it moved, not a model of money it
expects to move.
**Location.** `tax.ohio.gov`, Government → Resources → Casino. The page links every file under
`/static/`; the bytes are served from `dam.assets.ohio.gov/raw/upload/tax.ohio.gov/…`, which is the
mapping [Table SD-1](dot-sd1-school-district-taxes.md) already documented for a different division
of the same department.

**What it contains.** Every Ohio school district that received money, once per county it has
resident students in: county code, county name, IRN, district name, and the amount. Eighteen
half-yearly distributions, **August 2015 through January 2024**, which is nine complete state
fiscal years, FY2016 through FY2024. Between 979 and 1,044 districts a distribution and between
3,026 and 3,397 district×county rows.

This is the only per-district record in the corpus of education money that is not an appropriation.
A 33% tax on gross casino revenue, 34% of it to the county student fund, apportioned among the 88
counties by resident public school student population and then within each county among its
districts. [Ohio Constitution Article XV §6(C)(3)(b); R.C. 5753.02, 5753.03(D)(2), 5753.11]

**What it is not.** Not an appropriation, not a payment from the Department of Education and
Workforce, and not in the School Finance Payment Report. The only casino money in the department's
budget is the **casino operator settlement fund**, a different pot that supported preschool slots
by interagency transfer from FY2017 to FY2023. Searching the department's budget for casino money
finds that and nothing else, which is the wrong answer arrived at honestly.

**Why the series stops in January 2024.** Because the department's own page stops there. It is not
a retrieval failure and not a moved directory: the page lists a `County Student Distribution
Report` and an `HC Student Distribution` for every half-year from August 2015 to January 2024 and
carries nothing dated later, in any category. The distributions **before** August 2015 exist as
`Final SD Distribution` PDFs with no machine-readable twin, which is where the series begins.

**A district appears once per county, and that is the statute working.** R.C. 5753.03(D)(2)
apportions within a county among the districts with students resident in it, so a district that
draws from three counties is paid from three county funds. In January 2024, 294 of 1,001 districts
were in exactly one county and three were in all **88** — Ohio Connections Academy, Ohio Virtual
Academy and Ohio Distance Education, whose students live everywhere. The key is (county, IRN).

**Four traps, each of which produces a plausible number.**

**The file states its own total, in a row inside the data block.** `Total Distribution Amount` sits
in the key column with the total in the amount column, so a parser that sums the column without
filtering to a six-digit IRN reports **exactly twice** the distribution. Ninety million dollars for
a half-year has nothing obviously wrong about it. The same property is the reason this source can
be trusted: the sheet states an aggregate beside its parts, so
[`build_casino_extract`](../../crates/connect/src/fixtures/casino.rs) makes each of the eighteen check
itself rather than trusting a reviewer's sense of scale.

**The amount column is spelled `Distrubution Amount`** in twelve of the sixteen workbooks,
`Distribution Amount` in three, and `Total Distribution` in the August 2018 file.

**August 2018 is a true-up and has three amount columns.** A January 2018 recalculation of
**$7,475.78**, the August 2018 calculation of **$48,045,656.79**, and their total. The
recalculation sits in column E — where every other year's amount is — so a fixed offset reports the
half-year at seven thousand dollars.

**The analyst's copy is a different file at a neighbouring path.** `2024 01 County Student
Distribution Report.xlsx` and `…Report Web.xlsx` are both live; the first carries an extra
`RP_MAIN_PG1 (2)` sheet reconciling districts against county allocations, where the two differ by
four cents. The `Web` file is the one the page links, and it is the one pinned. The August 2021
file keeps its reconciliation in a `Sheet1`: 75 of the 88 counties balance and 13 are out by
exactly a cent.

**How much it is.** FY2016 $90.8m, FY2017 $89.4m, FY2018 $92.0m, FY2019 $93.9m, FY2020 $96.0m,
FY2021 $73.9m, FY2022 $109.4m, FY2023 $113.1m, FY2024 $114.2m, on a payment basis. The FY2021
figure is the pandemic closure, which arrives as one half-year: January–June 2020 came to
**$24.6m** against a floor of $42.9m across the other seventeen half-years, and the next half-year
was back to $49.3m.
