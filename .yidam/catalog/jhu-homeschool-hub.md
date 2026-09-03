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
47,491 (2021-22) → 47,468 (2022-23) → 53,051 (2023-24) → 61,009 (2024-25).

**Sheet `BY COUNTY, DISTRICT, TOWN, LEA` — 611 Ohio rows, one school year.** [inference] Values
like `Akron City, 373`, `Alliance City, 70`, `Beachwood City, <10`. This is the only per-district
home-education breakdown known to exist outside the department, and the four defects below are why
it is catalogued rather than committed.

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

## Four defects in the district sheet

**The year label contradicts the workbook's own state total.** Every Ohio row is labeled
`SY21-22`. Of the 611 rows, 564 carry a number and they sum to **51,349** — a floor, since the
remaining 47 are censored and each is at least zero. The same workbook's statewide sheet gives
2021-22 as 47,491, and a district file cannot exceed its own state total by 3,858. It reconciles
with the year before: 2020-21 is 51,502, leaving 153 across 47 censored cells, about 3.3 each,
which is what 27 `<10` cells and 20 `.` cells should sum to. The sheet is most likely SY2020-21
mislabeled [inference] — the pandemic peak, and the worst year in the series to mistake for a
typical one. Nothing in the workbook flags it, and which year it actually covers is `[open]`.

**No IRN, and 48 rows are unresolvable without one.** Districts are named, not keyed. 611 rows
carry 583 distinct names: 20 names repeat across 48 rows, some three times. `Perry Local` appears
as 15, 34 and 87 with nothing to say which is Lake, which is Stark and which is Franklin; the
same holds for `Buckeye Local` (44, 35, 91), `Springfield Local` (124, 18, 116), `Northwest
Local`, `Southern Local` and fifteen others. The sheet's schema carries a county column that other
states populate and Ohio's rows leave empty. Every district panel in this repository joins on IRN,
so those 48 rows have nowhere to land.

**47 censored cells in two undocumented flavours.** 27 `<10` and 20 `.`, about 8% of districts.
The workbook's codebook defines the five disaggregation categories and says nothing about either
marker, so the difference between a suppressed small count and an unreported one is a guess.
Unlike the report-card cells, there is no companion percentage column to recover a value from —
columns E and F are empty for every Ohio row.

**The provenance does not resolve.** The `DATA LINKS` sheet cites, for Ohio, the department's Home
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
everywhere and concentrated somewhere, and the aggregate is silent about which.

## Access constraints

The workbook exports cleanly as `.xlsx` and needs no credentials. The Hub's own state pages do
not: `education.jhu.edu` answers 403 to an automated client and to a browser user-agent alike, so
the surrounding methodology and the link to this file were read from search results rather than
from the page. Not pinned by digest and not fetched by a connector, and it should not be — a
spreadsheet export has no stable content address, so nothing here can rebuild byte-identically
from it. That is the whole reason the series is catalogued and not committed as a fixture.

**Nobody has looked at the other 38 states.** This entry was created for the Ohio rows and read
only those; the `All States` sheet, the four disaggregation sheets and every other state sheet are
unexamined.

## What would replace this entry

A records request to the department for notification counts by district and IRN, FY2006 through
FY2025, together with the suppression rule, the school year the released file actually covers, and
whether the count is notices or students. Tracked as
[#257](https://github.com/goedelsoup/ohio-education-funding/issues/257), in the same class as the
withdrawn per-district scholarship breakdown — published to somebody, not published to everybody,
with the difference that this one has never been public in any form.
