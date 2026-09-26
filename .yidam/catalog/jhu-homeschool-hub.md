# Johns Hopkins Homeschool Hub — state and district home-education counts

**Source.** Johns Hopkins University School of Education, Institute for Education Policy,
*Homeschool Hub*. A single workbook compiling homeschool participation counts collected by state
education agencies, one sheet per state plus five disaggregation sheets.
**Type.** Secondary source — an aggregator republishing counts the states collect. It is not the
source of record for any figure in it, and the distinction is load-bearing here.
**Location.** `docs.google.com/spreadsheets/d/1xFaTmfL1W-rkgrT_M39aApH2SVTjpnte/`, linked from
`education.jhu.edu/edpolicy/policy-research-initiatives/homeschool-hub/states/ohio/`.

**What it contains.** 47 sheets: an `All States` summary, 39 state sheets, a codebook, a data-links
sheet, and five sheets disaggregating by county/district, grade or age, gender, race, and family
structure. Ohio appears in exactly two of them, and in none of the last four.

## The two Ohio sheets

**Sheet `OH` — a statewide annual series, 2005-06 through 2024-25.** [inference] Blank for
1999-2000 through 2004-05. It runs 25,937 (2005-06) → 33,328 (2019-20) → 51,502 (2020-21) →
47,491 (2021-22) → 47,468 (2022-23) → 53,051 (2023-24) → 61,009 (2024-25). Committed as
`crates/project/fixtures/home-education-statewide.csv`, twenty rows, the blank years dropped.

**Sheet `BY COUNTY, DISTRICT, TOWN, LEA` — 611 Ohio rows, one school year.** [inference] Values
like `Akron City, 373`, `Alliance City, 70`, `Beachwood City, <10`. This is the only per-district
home-education breakdown known to exist outside the department. Committed as
`crates/dispersion/fixtures/home-education-districts.csv`, keyed by IRN, each count carried as a
floor and a ceiling. Three of the four defects recorded below are why it took until
[`home-education-connector`](../decisions/home-education-connector.yml) to get there, and three
of the four did not survive being checked.

## Why only one figure in it is [verified]

The prelude's rule is that where the source of record is unreachable and an aggregator carries the
same figures, those figures support `[inference]` and never `[verified]`, however good they are.
This entry is the case that rule describes, and the split is unusually clean.

Ohio publishes **one** home-education number anywhere in machine-reachable form:
`Home School — 53,051`, for 2023-24, in
[Ohio's Education Landscape 2023-2024](https://education.ohio.gov/getattachment/Topics/Data/Frequently-Requested-Data/Facts-and-Figures/Ohios-Education-Landscape-2023-2024.pdf.aspx?lang=en-US).
That figure is `[verified]` — retrieved and read directly from the department's own fact sheet,
and the workbook's 2023-24 cell matches it to the student, which is what identifies this series as
the department's numbers carried forward rather than an independent estimate.

**That fact sheet is now registered and pinned**, as `education-landscape-2024`, with its own
entry at [`dew-education-landscape`](dew-education-landscape.md) and its School Options table
committed as a fixture. When this entry was written the one verified figure in it rested on a
live URL and nothing else — which, given that every prior-year edition of the same sheet now
returns a 404, was one link-rot away from being unverifiable. It no longer is.

Every other year, and the whole district sheet, rests on the aggregator alone. The department's
[Home Schooling](https://education.ohio.gov/Topics/Ohio-Education-Options/Home-Schooling) topic
page carries no counts, its home-education fact sheet carries no counts, and the
[Enrollment Data](https://education.ohio.gov/Topics/Data/Frequently-Requested-Data/Enrollment-Data)
files — which reach back to 1978 for public districts and nonpublic buildings — have no
home-education breakout by district of attendance or of residence. The prior-year Landscape
editions do not exist to be fetched: 2018-19, 2019-20, 2020-21, 2021-22, 2022-23 and 2024-25 all
return an identical 1,245-byte 404, as does the older-format `2013-2014-WHERE-KIDS-COUNT` still
carried in search indexes, and both facts-and-figures index pages link the 2023-24 edition and
archive nothing. Whether the Internet Archive holds prior editions under the same URL pattern is
`[open]`; the probe was blocked in the session that wrote this entry.

## The four defects, and what became of them

**The year label contradicts the workbook's own state total. Standing, and it is the finding
rather than the defect.** Every Ohio row is labeled `SY21-22`. Of the 611 rows, 559 carry a number
and they sum to **51,349** — a floor, since the remaining 52 are censored and each is at least
zero. The same workbook's statewide sheet gives 2021-22 as 47,491, and a district file cannot
exceed its own state total by 3,858.

This entry first read that as 2020-21 mislabeled, on the arithmetic that 2020-21 is 51,502 and the
remaining 153 students spread over the censored cells at about 3.3 each. That assumed the censored
districts were small. They are not: Huber Heights is 5,631 pupils, Euclid 4,166, Southwest Local
4,150. Against contemporaneous F-33 enrolment, 2020-21 needs a censored participation rate of
0.22% against a reported 3.55% — a sixteenfold discontinuity — and 2021-22 and 2022-23 are
arithmetically impossible. A size-matched estimator on 2023-24 reconstructs **53,100** against the
published **53,051**, missing by 49 students where every other year misses by 3% to 13%. The sheet
is 2023-24 [inference]; the fixture carries the publisher's `SY21-22` verbatim in a
`published_year` column and writes no inferred year anywhere, and the builder fails if a revision
ever makes the two sheets agree.

**No IRN — but the rows are in IRN order, and all 611 resolve. Answered.** Districts are named,
not keyed, and 611 rows carry 583 distinct names: 20 names repeat across 48 rows, some three
times. `Perry Local` appears as 15, 34 and 87 with nothing to say which is Lake, which is Stark
and which is Franklin; the same holds for `Buckeye Local` (44, 35, 91), `Springfield Local` (124,
18, 116), `Northwest Local`, `Southern Local` and fifteen others. The sheet's schema carries a
county column that other states populate and Ohio's rows leave empty.

What nothing had checked is the row order. 560 of the 611 names are unique in the department's own
district list, and 558 of those strictly increase by IRN — one monotone run covering rows 1
through 606, with four rows appended after it. A repeated name lying between two unique ones is
bounded by their IRNs, and the bound admits exactly one candidate in every case but four; those
four fall out by elimination against the IRNs already taken. **611 rows, 611 distinct IRNs,
nothing unresolved**, and the resolution is a committed test rather than a hand-built crosswalk.

**Three censoring markers, not two, and the codebook documents none.** 27 `<10`, 20 `.` and **5
`NC`** — 52 cells, about 8.5% of districts. The two-marker count in the first draft of this entry
came from subtracting 47 from 611 and never looking. The workbook's codebook defines the five
disaggregation categories and says nothing about any of the three, so the difference between a
suppressed small count, an unreported one and a not-collected one is a guess. Unlike the
report-card cells, there is no companion percentage column to recover a value from — columns E
and F are empty for every Ohio row. Nine other states use `.` and Ohio alone uses `<10` and `NC`,
which is evidence for reading the first as the compiler's blank and the other two as the
department's own markers, and evidence is not documentation: the fixture carries all three
verbatim. `<10` bounds a cell at `[0, 9]`; `.` and `NC` get a floor of zero and **no ceiling**,
because nothing in the file bounds them above and one of them is a 4,150-pupil district.

**Twenty districts report an explicit `0.0`,** which no defect in the original four covers and
which is the reason the markers have to stay distinguishable. Among them is Cincinnati Public
Schools, 34,860 pupils, the third-largest district in the state. A published zero and a
`.` are different claims and would be indistinguishable under any imputation.

**The provenance does not resolve. Standing, and unfixable from here.** The `DATA LINKS` sheet cites, for Ohio, the department's Home
Schooling topic page — the page that publishes no counts at all. The file's own citation does not
serve the file. The release route was presumably a records request; the workbook gives no date, no
requester, and no definition of what is being counted. Whether the measure is notices received or
students named on them is `[open]`, and the two differ by household size.

## What the department collects and does not publish

EMIS carries the flow but not the stock.
[EMIS Manual §2.4, Student Standing (FS) Record, v14.1](https://education.ohio.gov/getattachment/Topics/Data/EMIS/EMIS-Documentation/Current-EMIS-Manual/2-4-Student-Standing-FS-Record-v14-1.pdf.aspx?lang=en-US)
defines Withdrawal Reason `43 — Transferred to Home Education`, "Parent or guardian notice on
file". [verified] It is reported per district and published per district nowhere, and on its own
it is the wrong measure: a child who begins home education at six and never enrols is invisible to
it permanently. The stock lives in the R.C. 3321.042 notices held by each district of residence,
which no statute requires anyone to forward to the state as a count. [verified]

## Why the figures matter to this corpus

Home-educated students carry no ADM, so they leave the formula without appearing in any enrollment
file. The only fiscal trace anywhere in this repository is a $250,000 FY2015 earmark inside GRF
200550 for PSEO on behalf of home-schooled students, in the H.B. 59 greenbook. [verified] At
61,009 the population is larger than every scholarship programme in the department's 2025
[annual report](dew-scholarship-annual-report.md) but one, and larger than the JVSD enrollment the
Landscape fact sheet prints beside it.

The question it bears on is a denominator question — how much of a district's resident school-age
population has left the public system by a route no enrollment file records — and a statewide
total cannot answer it, for the same reason the statewide scholarship totals cannot close the
per-district gap in that report. A population spread across some 600 districts is thin almost
everywhere and concentrated somewhere, and the aggregate is silent about which. The district
fixture is what that question is now asked against, for one school year and with 52 cells masked.

## Access constraints

The workbook exports cleanly as `.xlsx` and needs no credentials. The Hub's own state pages do
not: `education.jhu.edu` answers 403 to an automated client and to a browser user-agent alike, so
the surrounding methodology and the link to this file were read from search results rather than
from the page.

**The export is a stable content address.** The first draft of this entry asserted that it could
not be, and that assertion was a supposition nobody had measured. Four downloads over two days
returned the same 645,110 bytes under the same SHA-256, so the file is now fetched by
`jhu-homeschool-hub`, pinned in `crates/connect/source-digests.txt`, and both fixtures rebuild
byte-identically from it. Google serves the export deterministically for an unedited sheet; an
edit to the sheet would change the digest, which is what a digest is for.

**Nobody has looked at the other 38 states.** This entry was created for the Ohio rows and read
only those; the `All States` sheet, the four disaggregation sheets and every other state sheet are
unexamined.

## What would replace this entry

A records request to the department for notification counts by district and IRN, FY2006 through
FY2025, together with the suppression rule, the school year the released file actually covers, and
whether the count is notices or students. The committed fixtures answer the *which district* half
and leave the other three open: one year, three undocumented markers, and an undefined measure.
Tracked as
[#257](https://github.com/goedelsoup/ohio-education-funding/issues/257), in the same class as the
per-district scholarship breakdown the annual report cites and this project has not reached —
published to somebody, not published to everybody, with the difference that this one has never
been public in any form.
