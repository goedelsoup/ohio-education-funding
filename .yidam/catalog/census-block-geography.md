# Census block geography: school districts, legislative districts, and population

**Source.** U.S. Census Bureau, Geography Division and Redistricting Data Office. Three files:
`BlockAssign_ST39_OH.zip` (2020 Block Assignment File), `sldl24.zip` (2024 State Legislative
District Block Equivalency Files), `oh2020.pl.zip` (2020 Census Redistricting Data, P.L. 94-171).
**Type.** Primary source — the Bureau's own record of which geographies contain which of Ohio's
276,428 census blocks, and how many people live in each.
**Location.** `www2.census.gov/geo/docs/maps-data/data/baf2020/`,
`www2.census.gov/programs-surveys/decennial/rdo/mapping-files/2025/2024-state-legislative-bef/`,
and `www2.census.gov/programs-surveys/decennial/2020/data/01-Redistricting_File--PL_94-171/Ohio/`.

**What they contain.** The block assignment file maps each Ohio census block to the geographies
containing it — among them `SDUNI`, the unified school district. The block equivalency file maps
the same blocks to the state legislative districts of the **2024** plan. The redistricting file
carries each block's population and its population aged 18 and over; the difference is the
under-18 count, 2,591,886 for Ohio, 22.0% of the state.

**Why this is here.** Ohio's school funding system contains no legislative district. The department
computes funding per school district and stops, so there is no published mapping from school
districts to House districts and no published funding figure for one — not because it is secret but
because nobody in the system has ever needed the number.

Building it needs block-level geography because school districts and House districts do not nest.
**339 of Ohio's 609 school districts have population in two or more of the 99 House districts**;
Columbus has population in eleven. County can be an attribution — the department assigns each
district exactly one, and the corpus inherits it. A House district cannot.

**The trap in the first file, which is worth stating plainly.** `BlockAssign_ST39_OH.zip` contains
its own `BlockAssign_ST39_OH_SLDL.txt`, sitting in the same archive as the school district file and
looking exactly like the right input. It is the **2020-cycle** map. Ohio redistricted afterwards,
and **66.3% of Ohio's census blocks are in a different House district under the plan now in use**.
A crosswalk built from the convenient file would have been wrong for two-thirds of the state and
would have looked entirely plausible — 99 districts, every school district placed, totals
reconciling. Hence the separate 2024 source, which exists because Ohio is one of eight states with
changes to both chambers in that cycle.

**What the corpus does with them.** Joins all three on the block identifier, aggregates to
`(school district, House district)` pairs, and expresses each pair as a share of the school
district's under-18 population. That is
[`crates/project/fixtures/house-district-crosswalk.csv`](../../crates/project/fixtures/house-district-crosswalk.csv),
1,085 rows covering all 609 districts in the funding panel with no losses.

The join to Ohio's own identifiers runs through the [NCES Common Core of
Data](nces-ccd-lea-directory.md) directory, whose `ST_LEAID` column is the IRN.

**What it does not support.** Anything stated as a fact about a House district's school funding.
Every apportioned figure is an estimate: the weight is under-18 population, which is a proxy for
pupils and counts children in community schools, private schools and none; and it is a **2020**
count applied to a **FY2027** model, so a district that has grown or emptied since is weighted as
it was. The one exact property is conservation — each district's shares sum to one, so the 99
House districts sum to the statewide total to the cent.

**Vintage risk the header assertion cannot catch.** The crosswalk is pinned to the 2024 map. When
Ohio redistricts again the file at the same URL changes and the fixture becomes silently stale: the
column names do not move, so `project::house_district`'s header check will pass. The digest manifest
is what detects it.
