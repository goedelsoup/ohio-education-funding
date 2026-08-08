# Annual Survey of School System Finances (F-33)

**Source.** U.S. Census Bureau, Economic Reporting. `elsec22t.xls`, the FY2022 individual unit
table from the Public Elementary–Secondary Education Finance Data series.
**Type.** Primary source — a census of school systems, compiled from state education agency
reporting on the Bureau's own definitions rather than each state's.
**Location.** `www2.census.gov/programs-surveys/school-finances/tables/2022/secondary-education-finance/`.

**What it contains.** Every school system in the United States, one row each: 14,106 of them,
with fall enrolment, revenue by source (federal, state, local, and within local the property tax
and the parent-government appropriation separately), current spending broken into functions,
capital outlay, debt, and per-pupil derivations. Seventy-two columns.

**Why this is the only source here that can answer the question it answers.** Every other source
in this catalog is Ohio describing itself. That is enough to say what Ohio does and not enough to
say whether it is unusual — and the central holding of
[*DeRolph*](derolph-litigation-record.md) is a claim of exactly the second kind: that Ohio relies
*too heavily* on local property tax. "Too heavily" is a comparison and the corpus had nothing to
compare against. This is that comparison, on one set of definitions applied to all fifty states.

The answer is that Ohio is unusual and in the direction the court said. Ohio raises **51.8% of
school revenue locally against a national 43.4%, seventh highest of fifty-one**, and takes
**34.4% from the state against a national 43.4%, forty-fifth of fifty-one**. Its spending per
pupil is $14,923 against a national $15,801 — twenty-fourth, unremarkable — and its federal share
is within a point of the national figure. What distinguishes Ohio is not what its schools cost
but who pays.

## Two traps in this file, both of which produce a confident wrong answer

**`STATE` is not a FIPS code.** Column 0 is the Census Bureau's own state ordering and column 4
is FIPS. They agree for the alphabetically early states and diverge after: filtering column 0 for
`39` returns Pennsylvania, whose FIPS is 42. Both are two-digit and zero-padded, so the error is
invisible — the first run of this connector reported Ohio figures that were Pennsylvania's.

**Nine states report zero school property tax and levy plenty.** Alaska, Connecticut, the
District of Columbia, Maryland, Massachusetts, North Carolina, Rhode Island, Tennessee and
Virginia have *dependent* school districts: the district is an agency of a city or county, so the
property tax belongs to the parent government and reaches the school system as an appropriation
(`LOCRPAR`). Virginia's parent contributions are 94% of its local school revenue, the District of
Columbia's 99%.

So `LOCRPROP / TOTALREV` is **not** a national ranking. It compares states that report their own
levy against states structurally unable to, and puts two of the most property-tax-dependent
states in the country at the bottom. `TLOCREV` includes the parent appropriation and is
comparable across both structures; rank on that, and report the property tax rank over the
thirty-nine independent-district states with the exclusion named.

## What the fixture holds, and what it does not

[`census-f33-states.csv`](../../crates/dispersion/fixtures/census-f33-states.csv) is the state
aggregate: fifty-one rows out of fourteen thousand, carrying enrolment, revenue by source,
property tax, parent-government appropriations and current spending. Money is in **thousands of
dollars**, as the survey reports it.

Systems are included when enrolment exceeds zero. That rule rather than a school-level filter,
because states organise differently — Ohio has 609 unified districts and no elementary-only ones,
Illinois has hundreds of both — and any rule stated in school levels would count different things
in different states. It admits everything that teaches somebody and excludes the two categories
that would double count: 691 education service agencies, whose revenue arrives *from* the
districts they serve, and 121 nonoperating systems, which levy tax and pay tuition elsewhere.

**The per-district panel is not extracted.** The survey identifies districts by NCESID and this
corpus keys on IRN, and no crosswalk between them is held here. That blocks the second use the
connector was declared for — an independent federal check on the department's own per-district
figures — and it is a retrieval problem rather than a parsing one.

**FY2022 is the peak of federal pandemic relief.** The federal share is inflated and the state
and local shares correspondingly deflated. Every finding above therefore understates Ohio's local
reliance rather than overstating it, which is the safe direction, but a series across years would
need the relief years marked. One year per file and the layout is not stable across them, so the
column map is per-era.
