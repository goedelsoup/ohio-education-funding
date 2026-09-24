# EdChoice Designated List — which buildings' students may claim a scholarship

**Source.** Ohio Department of Education and Workforce, *Designated List 2026-2027, With
Criteria*.
**Type.** Primary source, published by the programme administrator. An eligibility determination
rather than a report: this file *is* the designation, not a description of one.
**Location.** `https://education.ohio.gov/getattachment/Topics/Other-Resources/Scholarships/EdChoice-Scholarship/EdChoice-Resources/Designated-List-2026-2027-With-Criteria.xlsx.aspx?lang=en-US`.
876 KB, eight sheets, retrievable by a self-identifying agent with no credentials. Registered as
`edchoice-designated-2627` and pinned by SHA-256.

**What it contains.** The `Overview` sheet is the list: **2,877 buildings across 606 districts**,
keyed on district IRN and building IRN, each carrying its designation and every input to it — the
two statutory criteria, three years of bottom-20% Performance Index flags, three years of Title I
formula shares and their average, an academic-distress flag, grade levels served and an
open/closed status.

**513 buildings in 78 districts are designated for 2026-2027.** So eligibility is concentrated:
one district in eight has a designated building at all, and the other 528 have none.

Six further sheets carry the criteria's own inputs, and all six are extracted. `PI Ranking 2023`,
`2024` and `2025` rank every building the department ranks by Performance Index — **9,080 rows, a
three-year building-level panel** at the precision the ranking is actually decided on, where the
report card carries one year at one decimal place. `Title 1 2024`, `2025` and `2026` give each of
**611 districts** a Title I formula count decomposed into census poverty, neglected, delinquent,
foster and TANF children against its 5-to-17 population — **the first held source in this
repository for Ohio's Title I formula counts at all**.

Together they make the whole determination reproducible from raw counts rather than from the
department's own flags. It reproduces, exactly, in all 2,877 rows. What that costs is three
readings the file states nowhere, and the third of them is a finding about the file: see below.

**Why it matters here.** It is the **public substitute for the
`nonpublic-data-historical-ed-choice-designated-list` route** recorded in
[`deduction`](../skills/deduction.md) — a portal route this project has not reached, once read
here as withdrawn and not shown to be: that host answers every path with the same application
shell, so its 404 says nothing about the report
([#10](https://github.com/goedelsoup/ohio-education-funding/issues/10)). And it is the first
per-district file in this repository about the scholarship channel. It does not replace what that
route carries: this is who *may* claim, not who did, so it answers a different question — but it
answers it per district, which nothing else here does.

## One column is a dated fact, and the date is in the file

The workbook's own `docProps/core.xml` gives `dcterms:created` and `dcterms:modified` as
**13 November 2025**, thirty-four minutes apart — made in one sitting, unrevised since, and the live
file is still byte-identical to it as of 15 September 2026.

That date matters for exactly one of the twenty columns. Every other is an arithmetic on the six
sheets beside it, so it is as current as its inputs and cannot drift. `Academic Distress School`
is not: R.C. 3310.03(C) makes a student eligible "if the student's resident district is subject to
section 3302.10", so the column records whether a commission exists, and nothing in the file
computes it.

**It is maintained, and Lorain proves it.** Three districts have held a commission under the current
R.C. 3302.10. Lorain City (IRN `044263`, fifteen buildings — not `047076`, which is Pettisville
Local) is flagged `no`, its commission having been dissolved by H.B. 33 on 4 July 2023. So the
department recomputes this column rather than carrying it forward.

**And it went stale six weeks after it was written.** East Cleveland City (IRN `043901`, five
buildings) is flagged `yes`. In late December 2025 the director released the district from
commission oversight, and R.C. 3302.10(N)(1) ends the commission with the transition period — so
R.C. 3310.03(E)(2) had already stopped first-time Option A awards there before the 2026-2027 school
year this list governs opened. See
[`dew-academic-distress-commission`](dew-academic-distress-commission.md) for the notice and its
dating.

**What the staleness costs is nothing, and that is worth stating as precisely as the defect.** All
five East Cleveland buildings also satisfy Option B — bottom twenty per cent in two of three
rankings, Title I average 48.9% — so they are designated on the derived route regardless. Recomputing
the designation while honouring Option A only for Youngstown leaves all 513 designations standing.
The entire load-bearing surface of the unreproduced column is **two buildings**, both Youngstown's
Rayen Early College schools, and Youngstown's commission is the one that still exists.

Pinned in `crates/dispersion/tests/the_one_column_the_designation_reads_is_a_dated_fact.rs`, which
also records why reproducing R.C. 3302.10(A)(1)'s trigger would not audit this column: the trigger
predicts *establishment* and the column reports *existence*. East Cleveland is the demonstration —
released on an overall three stars, well above the trigger, and flagged `yes` anyway.

## The designation is checkable, and the check finds a rule the file does not state

`DESIGNATED` is exactly `Option A OR Option B` — 2 buildings by Option A alone, 495 by Option B
alone, 16 by both, and no building designated by neither. Option A is the academic-distress route
and is identical to the `Academic Distress School` column in all 2,877 rows.

Option B's criteria are R.C. 3310.03(A)(1): bottom-20% Performance Index "for at least two of the
three most recent consecutive rankings", and a district in which "an average of twenty per cent or
more" of its students were in the Title I formula over three consecutive years. Taken literally,
that rule disagrees with the department's own Option B column on **four buildings** — Highview 6th
Grade Center in Middletown City, Willyard Elementary in Ravenna City, Ripley Union Lewis Huntington
Junior High, and Beallsville High School in Switzerland of Ohio Local. All four meet both stated criteria and
all four are marked `No`.

All four are **Closed or Inactive**. There are fifteen such buildings in the file and not one of
them is designated. Adding that condition makes the identity exact in all 2,877 rows:

> Option B = bottom-20% PI in two of three years **and** Title I ≥20% across three years **and**
> the building is open.

The workbook applies that third condition and states it nowhere. It is not in the criteria the
statute lists either — it follows from the premise in front of them, that the student "is enrolled
in a school building operated by the student's resident district", which a closed building has
nobody to satisfy. Pinned in `crates/dispersion/tests/` rather than left as a reading.

## The 2025 ranking sheet is not the population the 2025 flag was cut from

The bottom-20% flag is `ceil(0.20 × N)` of the buildings R.C. 3310.03(A)(1)(a) admits — those
"operated by city, local, and exempted village school districts", the pilot-project district's
excluded. In 2023 and 2024 that is the sheet itself: neither holds a community school, a STEM
school or a Cleveland building, so a fifth of the sheet is a fifth of the eligible population and
the flags follow.

The 2025 sheet is different. It ranks 3,237 buildings, **359 of which the statute excludes** —
258 community schools, 8 STEM schools and Cleveland's other 93 — across 256 LEAs that appear in
neither of the other two years, and a community school holds rank 4. A bottom fifth of it flags
646 buildings and **misses 163 of the 576** the department flags.

The department's flag is right all the same: 576 is a fifth of 2,877, which is exactly the sheet
filtered to `Public School` outside Cleveland and carrying an index. So the ranking was done on
the statutory population and published on a wider one. That is the same defect as the Option B
column above — a rule applied more narrowly than the file shows — and it is the reason a reader
taking the sheet at face value cannot reproduce the most recent of the three flags.

## The Title I criterion is a census poverty rate under another name

The `Title 1` sheets decompose the count R.C. 3310.03(A)(1)(b) tests, and **96.7% of it, across
three years and 611 districts, is census poverty** — the Census Bureau's estimate, dated in each
sheet's own column headings to two years before the school year it decides and paired with an
October collection one year behind. The oldest of the three editions rests on a 2021 estimate.

The four categories Ohio collects itself come to 3.3%. Two barely exist: the delinquent column is
never a positive number in any of the three editions, and TANF falls from 930 children statewide
to none at all between the first and the second, unannounced. But because they can only add, they
can only move a district *over* the twenty per cent bar — and they move 16 districts over it, six
of which hold designated buildings. **31 designations rest on the 3.3%.**

## The Title I sheets carry five districts the list does not, for two different reasons

611 against the Overview's 606 and the report card's 607. Cleveland Municipal accounts for one and
is the statutory exclusion described below — it is ranked in 2025 and report-carded like any
district, and stands sixth of 611 on the criterion it is excluded from.

The other four are absent from every ranking sheet and from the report card as well: **Kelleys
Island Local, Middle Bass Local, North Bass Local and College Corner Local**. That is not an
exclusion. A Title I formula count is a count of resident children and needs no building, so a
district that operates none the department rates still has a count — North Bass Local carries a
row and a 5-to-17 population of zero. Reporting "five districts are missing" as one fact would
merge a statutory exclusion with an absence of buildings.

## The Title I column tests an average, and its name says otherwise

`Title 1 Percent 20% or Above Across 3 Years` reads like a condition on each of the three years. It
is not: it is `Title 1 Average >= 0.20`, in all 2,877 rows, which is exactly what
R.C. 3310.03(A)(1)(b) requires — "an average of twenty per cent or more". Reading the column name
literally instead misclassifies **164 buildings**, all of them districts that clear the bar on
average and fall under it in one year.

`Title 1 Average` is itself the mean of the three yearly shares beside it, to the last place, in
every row; and it is constant within a district, because it is a district figure repeated onto each
of that district's buildings rather than a building one. That is why the criterion is a district
one and the designation is a building one: two buildings in the same district can differ only
through their Performance Index.

## Cleveland Municipal is the one district that is not in it

606 districts, against the 607 the report card carries. The missing one is **Cleveland Municipal**,
and the reason is statutory rather than clerical. R.C. 3310.03 opens by making a student eligible
only if "the student's resident district is not a school district in which the pilot project
scholarship program is operating", and closes division (A)(1) by directing that "when ranking
school buildings … the department shall not include buildings operated by" such a district.
Cleveland is the pilot project district — it has a scholarship programme of its own, catalogued in this corpus
as `cleveland-scholarship` — so no Cleveland building can be designated, and the department's file
does not rank them at all.

The exclusion is therefore observable: a statutory sentence and a row count that agree. Pinned
alongside the identity above.

## What it is not

**It is not participation.** A designated building is one whose students *may* apply. Nothing here
says a scholarship was awarded, used, or charged anywhere, and the portal route that would have
said so is still unreached — a report behind an entitlement, not a file that was taken down.

**It is not EdChoice Expansion.** Expansion eligibility is income-based and turns on no building at
all, so a designated list has nothing to do with it. This file bears on the traditional programme
only, which is the smaller of the two: 42,607 students against 100,939 in 2024-25. Note that
R.C. 3310.032 carries the *same* pilot-project exclusion, so Cleveland's absence here is not the
whole of it — a Cleveland resident is outside both EdChoice programmes, not merely outside the
building-based one.

**It is not the whole series.** Three editions are held — 2024-2025, 2025-2026 and 2026-2027 —
and 2023-2024 is named in the archive and not retrievable. See below.

## The page lists one edition at a time, and the archive says so too

The [EdChoice Resources](https://education.ohio.gov/Topics/Other-Resources/Scholarships/EdChoice-Scholarship/EdChoice-Resources)
page was read in full on 24 September 2026 rather than probed for guessed filenames. It carries
**47 `getattachment` links and exactly one designated list**: the 2026-2027 edition. The other 46
are forms, fact sheets and guidance, most of them in seven languages. There is no archive section,
no "prior years" list, and no second workbook of any vintage.

Two corrections to the guess that opened this question. The prior editions spell `with` in **lower
case** — `Designated-List-2025-2026-with-Criteria.xlsx` — and only 2026-2027 capitalizes it, so a
probe built from the current filename's pattern was testing a name the department never used.
Under either spelling all three predecessors return this host's genuine 404, a 1,245-byte page,
**byte-identical across all three** and distinguishable from the reports portal's application
shell.

**So the department is not renaming the file; it is deleting it.** That is the finding, and it is
one the previous text could not make: the listing carries one edition because the page carries one
edition, and the URL a departed edition used answers with a real 404 rather than a redirect.

The Wayback Machine holds the page 17 times between 30 November 2023 and 16 April 2026, and the
workbooks themselves:

| Edition | Captures | Held |
| --- | --- | --- |
| 2023-2024 | none of the file; linked from page captures only | no |
| 2024-2025 | `20240928123347`, `20250309093228`, then 404 from `20250714111157` | yes |
| 2025-2026 | `20250309082659`, `20250714111029`, `20250717154725`, `20250901074111` | yes |
| 2026-2027 | `20260120123937` | served live |

Every capture of a given edition is **byte-identical to every other** — one SHA-256 per edition
across four captures and three — so an edition is a fixed file and the earliest capture is the
whole of it. The 9 March 2025 captures also date the deletion: both 2024-2025 and 2025-2026 were
being served that day, and 2024-2025 was gone by 14 July. A fourth edition is therefore named and
not retrievable: 2023-2024 is linked from captures of the page and was never itself crawled.

Registered as `edchoice-designated-2526` and `edchoice-designated-2425` under
`dew-scholarship-reports`, pinned by SHA-256, and admissible under
[`an-archived-source-is-still-a-source`](../decisions/an-archived-source-is-still-a-source.yml):
the publisher no longer serves them, the reason is its release model rather than a correction, and
each **reproduces the department's own arithmetic in every one of its rows** — the designation
from its two options, Option B from its criteria including the unstated open condition, the window
flag from its year columns, and the Title I average from the three shares beside it.

## Two editions make eligibility a change rather than a snapshot

**460 designations, then 494, then 513**, over 2,937, 2,906 and 2,877 listed buildings, in 62, 68
and 78 districts. All three editions carry 606 districts, so the pilot-project exclusion is
constant and the growth is in designations, not coverage.

**Every building that entered the designated set entered through Option B.** 42 entries in
2025-2026 and 38 in 2026-2027, and not one of the 80 holds Option A. Departures are 8 and 19, of
which exactly one across the two years held the academic-distress route. So the commissions are
nearly inert as a *margin*: the route that moves buildings in and out is the derived one, three
years running.

Entries and departures are counted apart from delistings, which is what keeping all 2,877 rows
buys: a building that stops being designated and a building that stops being listed are different
events, and only 1 departure in each transition is the second kind.

The pair also settles the open-building condition above. On one edition it was a rule inferred
from four rows; on three it is the department's practice, applied on 1, 7 and 4 rows, with no
closed or inactive building designated in any edition.

## Two layout findings the editions disagree on

**2024-2025 ranks two Performance Index years, not three, and its own heading says so.**
R.C. 3310.03(A)(1)(a) asks for the bottom twenty per cent "for at least two of the three most
recent consecutive rankings"; that edition's column reads `Bottom 20% PI Ranking Across 2022 and
2023 School Years` against `Across Two of Prior Three Years` in both later editions. Two of two is
a materially easier test than two of three, and the fixture keeps the difference in the column
name — `bottom_20_pi_two_of_two` — rather than flattening it.

**A year a building was not ranked is blank, not `No`** — 25 cells in 2024-2025 and 42 in
2025-2026, online academies, early learning centres and buildings that had not opened. The
2026-2027 edition writes `No` in all three of its year columns and leaves nothing blank, so this
is a distinction the department stopped making rather than one it never made. The blank survives
into the fixture, because a building that was outside the ranking is not a building the ranking
cleared. The window column reads it as a year outside the bottom fifth, in every row of both
editions.

**And the missing space is new.** `Title 1Formula Average` in the 2026-2027 Option B heading is
this edition's alone: both archived editions spell it `Title 1 Formula Average`.

Pinned in `crates/dispersion/tests/the_editions_the_page_stopped_listing.rs`.
