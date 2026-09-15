# Historical Scholarship Data — the deduct era, counted two ways

**Source.** Ohio Department of Education and Workforce, *Historical Scholarship Data*, a workbook
prepared for posting 23 October 2018 and last touched 2 March 2022.
**Type.** Primary source, published by the programme administrator. An archival extract rather
than a report: the department's own note says it was taken from "archive of historical scholarship
application and usage data prior to the creation of the new Enterprise Application System", and
that the figures "reflect the data business rules used at the time of archival".
**Location.** `https://education.ohio.gov/getattachment/Topics/Other-Resources/Scholarships/Historical-Information/Historical-Scholarship-Data.xlsx.aspx?lang=en-US`.
21 KB, six sheets, retrievable by a self-identifying agent with no credentials. Registered as
`scholarship-historical` and pinned by SHA-256.

**What it contains.** Statewide participation by programme and fiscal year, on four sheets.

| Programme | Years | What is counted |
|---|---|---|
| Cleveland | FY1997–FY2013 | Applications and payments, each split new/renewal and low-income; plus tutoring grants |
| EdChoice (Traditional) | FY2007–FY2013 | Applications and payments, each split new/renewal and low-income |
| Autism | FY2004–FY2013 | Payments only, one column |
| JPSN | FY2013 | Applications and payments, total and new |

Two further sheets carry the department's own notes and its four income-status definitions —
*Qualified*, *Not Qualified – Over Income*, *Not Qualified – Incomplete*, *Not Started* — which is
the vocabulary behind the low-income columns and is worth reading before treating a blank one as a
zero.

**Why it matters here.** Two reasons, and the second is the larger one.

It is **the deduct era**, which nothing else committed reaches. The
[2025 annual report](dew-scholarship-annual-report.md) is entirely 2024-25, under the Fair School
Funding Plan, where every scholarship is its own funding unit paid directly. This file covers the
years when a scholarship was deducted from a resident district's foundation payment, and
[`deduction`](../skills/deduction.md) recorded that era as a retrieval problem no phase had solved.

And it publishes **two denominators for the same programme-year**. *Application Count* and
*Scholarships Used (At least 1 payment made)* are separate blocks on every sheet that has both, and
they are far apart: Cleveland's applications exceed its payments in every one of its seventeen
years, by **3.1×** in FY1997 and still by 11% in FY2013, and traditional EdChoice's by 9% to 19%.
The annual report's four published averages include three that are not expenditure over
participation, and that record guesses the gap is a denominator the report does not name. This is
the first held source showing the department itself maintaining two.

## The renewal column is not a count of applications

Recorded because it is visible in the file's own arithmetic and stated nowhere in it.

For Cleveland, **applications-renewal in year *t* equals payments-total in year *t*−1, exactly,
for twelve consecutive years** — FY1998 through FY2009. FY1998's 1,994 renewals are FY1997's 1,994
paid scholarships; FY2009's 6,272 are FY2008's 6,272. Twelve exact matches is not a coincidence
about family behaviour. It is a roll-forward: everyone paid last year was carried into this year's
renewal count administratively, whether or not a family did anything.

**It breaks cleanly at FY2010**, the same year the *Renewal and Qualified as Low Income* column
starts carrying values at all. From FY2010 renewals are counted independently and land below the
prior year's payments every time — −9.7%, −13.3%, −13.4%, −14.2% — which is attrition the earlier
convention made arithmetically invisible.

So a Cleveland "application" before FY2010 is two different things in one column, and the series
has a definitional break in the middle of it that no annotation marks. The break is pinned in
`crates/project/tests/`; the fixture carries the published numbers unaltered.

**The same identity does not hold for traditional EdChoice**, whose renewals never match the prior
year's payments in any year. Whatever the convention was, it was Cleveland's.

## What it is not

**It is not per district.** Every figure is statewide, which the department's note says outright —
the archive is "only available in aggregate for the state". So it does not close
[#10](https://github.com/goedelsoup/ohio-education-funding/issues/10), and the withdrawn
per-district routes recorded in [`deduction`](../skills/deduction.md) are still withdrawn.

**It is not money.** There is no expenditure, award or per-pupil figure anywhere in the workbook.
Sizing the deduct-era channel in dollars still needs a different source.

**It stops at FY2013, and what continues it is not a series.** The annual report picks the channel
up again at 2024-25. Between them the only participation figures this repository holds are the ones
LSC quotes in passing — FY2014 for all four programmes in the 131st's greenbook, FY2018 and FY2019
in the 133rd's, three of the four prefixed "about" and none saying which denominator it counts on.
Those bound the hole; they do not fill it.

**Blank and `NA` are different things the fixture cannot tell apart.** Cleveland's tutoring column
reads `NA` from FY2010 — the department saying the programme's figures are not available — while
the low-income columns before FY2010 are simply empty. Both reach the fixture as absent, because a
CSV has one way of saying nothing, and neither becomes a zero.
