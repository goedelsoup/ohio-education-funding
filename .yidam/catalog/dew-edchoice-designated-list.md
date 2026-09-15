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

Six further sheets carry the criteria's own inputs, which this repository holds and does not yet
extract: `PI Ranking 2023`, `2024` and `2025` rank every building in the state by Performance
Index — a three-year building-level PI panel — and `Title 1 2024`, `2025` and `2026` give each
district's Title I formula count decomposed into census poverty, neglected, delinquent, foster and
TANF children against its 5-to-17 population.

**Why it matters here.** It is the **public substitute for the withdrawn
`nonpublic-data-historical-ed-choice-designated-list` route** recorded in
[`deduction`](../skills/deduction.md), and the first per-district file in this repository about the
scholarship channel. It does not replace what that route carried: this is who *may* claim, not who
did, so it answers a different question — but it answers it per district, which nothing else here
does.

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
says a scholarship was awarded, used, or charged anywhere, and the withdrawn route that would have
said so is still withdrawn.

**It is not EdChoice Expansion.** Expansion eligibility is income-based and turns on no building at
all, so a designated list has nothing to do with it. This file bears on the traditional programme
only, which is the smaller of the two: 42,607 students against 100,939 in 2024-25. Note that
R.C. 3310.032 carries the *same* pilot-project exclusion, so Cleveland's absence here is not the
whole of it — a Cleveland resident is outside both EdChoice programmes, not merely outside the
building-based one.

**It is one edition, and the history is not at this URL.** `Designated-List-2025-2026-With-Criteria`,
`-2024-2025-`, `-2023-2024-` and the undecorated `Designated-List-2025-2026` all return this host's
genuine 404 — a 1,245-byte page, distinguishable from the reports portal's shell. Per the MR-81
precedent, a filename pattern going quiet is usually a naming change rather than a withdrawal, so
the prior editions are worth a scrape of the EdChoice Resources page before being called lost.
