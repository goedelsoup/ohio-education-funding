# Typology of Ohio School Districts — the department's own similar-district grouping

**Source.** Ohio Department of Education and Workforce, *2013 School District Typology*.
**Type.** Primary source, published by the classifying body. The file *is* the assignment, not a
description of one.
**Location.** `https://education.ohio.gov/getattachment/Topics/Data/Frequently-Requested-Data/Typology-of-Ohio-School-Districts/2013-School-District-Typology.xlsx.aspx`.
79 KB, two sheets, retrievable by a self-identifying agent with no credentials. Registered as
`district-typology-2013` and pinned by SHA-256.

**What it contains.** The `Typology By District` sheet assigns **614 districts** a code, keyed on
IRN, and carries the four measures the assignment was made on: enrollment, median household
income, student poverty and percent minority. The second sheet lists five exemplar districts per
code and is editorial illustration — every district on it is already on the first.

**Why it matters here.** `education-agency.ont.yml` has declared a `typology` property since the
ontology was written, and not one agency node could fill it. Five nodes characterise their
district in prose and then record the code itself as `unfilled:` — *"Rural, high poverty. Exact
department typology code not yet entered."* `.yidam/skills/scenario-delta.md` states the cost
plainly: **"Typology is not an available axis"**, tagged `[open]`. And
[`fordham-house-bridge-commentary`](fordham-house-bridge-commentary.md) warns that **assessed
valuation per pupil is the nearest proxy the corpus holds and it is not the department's
typology**. This is the source those four entries were waiting for.

## Eight codes, and they are ordered by urbanicity rather than by poverty

| Code | Category | Districts |
|---|---|---|
| 0 | Removed from the analysis | 5 |
| 1 | Rural — High Student Poverty & Small Student Population | 124 |
| 2 | Rural — Average Student Poverty & Very Small Student Population | 107 |
| 3 | Small Town — Low Student Poverty & Small Student Population | 111 |
| 4 | Small Town — High Student Poverty & Average Student Population Size | 89 |
| 5 | Suburban — Low Student Poverty & Average Student Population Size | 77 |
| 6 | Suburban — Very Low Student Poverty & Large Student Population | 46 |
| 7 | Urban — High Student Poverty & Average Student Population | 47 |
| 8 | Urban — Very High Student Poverty & Very Large Student Population | 8 |

**The sequence is not a poverty ranking, and reading it as one is the error this table exists to
prevent.** The department orders the codes by a location composite — least urban to most — and
poverty is the *secondary* term inside each locale. So median student poverty runs 46.4% at code
1, 36.7% at 2, 29.0% at 3, and then back **up to 51.5% at code 4**, above codes 1, 2, 3 and 5
alike. A chart banding districts 1→8 on an ordered ramp would be asserting a monotone that four
of the eight steps break. [verified]

## The vintage is eleven years old, and the file says so itself

The scheme was established in 1996, revised in 2007 for the 2000 census, and revised again in
2013 for the 2010 one. **Nothing has revised it since.** The workbook's own `docProps/core.xml`
gives `dcterms:modified` as **6 January 2015**, which corroborates the page's "amended January
2015" to the month and means the file has been untouched for eleven years. [verified]

That is a fact about the department rather than a defect here, and it has one consequence worth
stating in every place this data is used: a district coded *High Student Poverty* was one **in
2013**. The classification is current — it is the assignment in force — but the measures behind it
are not. This is why the extract carries all four of them under `_2013` names: a reader can set
`student_poverty_2013` beside the report card's current share and see the drift rather than having
to assume it away.

## Code zero is five districts the department declines to classify

`College Corner Local`, `Kelleys Island Local`, `Middle Bass Local`, `North Bass Local` and
`Put-In-Bay Local` — three islands, a border district and a second island — are coded `0`, which
the methodology calls *removed from the analysis*. They are the only five rows with any suppressed
measure, and the suppression is total for the two Bass Island districts: enrollment, income,
poverty and minority share all read `N/A`. [verified]

Three of the five are in this repository's FY2027 panel; `Middle Bass` and `North Bass` are not,
and that pair is the subject of its own finding on
[`fair-school-funding-plan`](../corpus/funding-regime/fair-school-funding-plan.yml).

Worth recording that the department reached the same conclusion about **Kelleys Island Local** —
four pupils — that this repository's own charting had to reach from the other direction: its
per-pupil figures are a different kind of quantity, and a scale fitted to them describes one
island rather than a state. [inference]

## It joins this repository's panel exactly

All **609** districts in the FY2027 panel carry a typology code. Not one is missing. The file's
five surplus rows are districts that have closed or merged since 2013 — `Bettsville Local`,
`Ledgemont Local`, `Newbury Local`, and the two Bass Island districts — which is the expected
shape of an eleven-year-old roster against a current one, and is the reverse of the failure mode
that would matter. [verified]

Of the 609: 3 at code 0, 123 at 1, 106 at 2, 110 at 3, 89 at 4, 77 at 5, 46 at 6, 47 at 7, 8 at 8.

## What it is not

It is not a locale code. NCES publishes an urban-centric locale (`ULOCAL`) for every district and
this is not that — it is Ohio's own composite, built from a location score crossed with poverty
and enrolment size. The two are not interchangeable and neither derives the other.

It is also not the CUPP similar-district grouping. The CUPP profile reports a district's
similar-district *averages* and never names the group, so the two files are complements: one says
which group, the other says what the group looks like.
