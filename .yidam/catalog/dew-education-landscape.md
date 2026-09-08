# Ohio's Education Landscape — the department's own count of every school option

**Source.** Ohio Department of Education and Workforce, *Ohio's Education Landscape 2023-2024*,
published 2025. The department's annual two-page statewide fact sheet.
**Type.** Primary source. The department counting the students it administers, not an aggregator
restating them.
**Location.** `https://education.ohio.gov/getattachment/Topics/Data/Frequently-Requested-Data/Facts-and-Figures/Ohios-Education-Landscape-2023-2024.pdf.aspx?lang=en-US`.
About 1.1 MB, two pages, retrievable without credentials. Registered as
`education-landscape-2024` and pinned by SHA-256.

**What it contains.** Statewide counts for 2023-2024: enrolment by race and by diverse-learner
category, district settings, school types, educator licensure and demographics, and — the table
this corpus takes — **School Options**, which counts every way an Ohio child is schooled other
than in a traditional district classroom.

| Channel | Enrolment |
|---|--:|
| Chartered Private Schools (711 of them) | 173,156 |
| Public Vouchers for Private School | 150,914 |
| — EdChoice Expansion | 88,095 |
| — EdChoice Scholarship | 41,234 |
| — Jon Peterson Special Needs | 8,551 |
| — Cleveland Scholarship | 8,028 |
| — Autism Scholarship | 5,006 |
| Community Schools | 116,973 |
| — Brick-and-Mortar | 83,959 |
| — Online (e-schools) | 33,014 |
| Inter-District Open Enrollment | 79,006 |
| **Home School** | **53,051** |
| Joint Vocational School Districts | 49,524 |

The sheet also prints **Total Enrollment 1,665,521** for the same year, which is the denominator
every share taken from this table is against.

## Why it matters here

**It is the only machine-reachable place the department publishes a home-education count.** The
Home Schooling topic page carries none, the home-education fact sheet carries none, and the
Enrollment Data files — which reach back to 1978 — have no home-education breakout by district of
attendance or of residence. That single cell is the whole of Ohio's published home-education
record, and everything else known about the series comes from an aggregator, which is why
[`jhu-homeschool-hub`](jhu-homeschool-hub.md) exists and why nothing in it can be `[verified]`.

**And it prints the channels side by side, counted the same way in the same year.** Every other
source here sizes one channel at a time, so comparing them means comparing publications with
different vintages, populations and definitions. This table does not have that problem, and it is
the only place in the corpus that does not.

## Two internal identities, and one cross-source check

The five voucher programmes sum to the voucher total exactly — 41,234 + 88,095 + 8,028 + 8,551 +
5,006 = 150,914 — and the two community-school types sum to theirs — 83,959 + 33,014 = 116,973.
Both are asserted where the fixture is read, because a layout change that shifted a column would
break them and nothing else would notice.

The five programmes also appear in the
[2025 Scholarship Annual Report](dew-scholarship-annual-report.md), for the *following* school
year. Every one is larger there, which is the corroboration that matters: two independent
department publications, consecutive years, the same five names, all moving the same way.

## What it does not carry

**Nothing per district.** Every figure is statewide, so this cannot answer where a channel is
concentrated — and the concentration is the interesting part for a funding question, because a
population spread across 600 districts is thin almost everywhere and heavy somewhere.

**One year.** The department replaces the sheet in place rather than archiving editions. The
2018-19 through 2022-23 and 2024-25 URLs all return an identical 1,245-byte 404, as does the
older-format `2013-2014-WHERE-KIDS-COUNT` still carried in search indexes, and both
facts-and-figures index pages link only the current edition. **This is therefore a series with one
retrievable member**, and the digest here is what stops that member from being lost the way its
predecessors were.

**No definitions.** The sheet gives counts and no methodology. Whether `Home School` counts
notices received or students named on them is not stated, and the two differ by household size —
the same ambiguity [`jhu-homeschool-hub`](jhu-homeschool-hub.md) records, unresolved from this
side too.
