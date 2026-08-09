# Common Core of Data: local education agency directory

**Source.** National Center for Education Statistics, U.S. Department of Education.
`ccd_lea_029_2223_w_1a_083023.csv`, the 2022–23 LEA universe directory.
**Type.** Primary source — the federal register of every public school district in the country,
compiled from state education agency reporting.
**Location.** `nces.ed.gov/ccd/data/zip/`.

**What it contains.** One row per local education agency, 1,058 of them in Ohio: the federal
`LEAID`, the state's own identifier in `ST_LEAID`, name, type, operational status, and address.

**Why this file, out of all of CCD.** The corpus has named `nces-ccd` as a connector since genesis
and left it `Declared`, blocked on the fact that a long identifier-change series — which districts
merged, split, or were renumbered across decades — has to be derived rather than read. That
remains true and remains unbuilt.

But it was blocking something much smaller that it never actually blocked. **Ohio's `ST_LEAID` is
the IRN behind an `OH-` prefix.** `OH-043786` is Cleveland Municipal. And the last five digits of
`LEAID` are the Census Bureau's unified school district code, so this one column is the join
between federal geography and Ohio's funding model.

Two connectors had been recording that join as unavailable. [F-33](census-f33-school-system-finances.md)
carried "the per-district join needs an NCESID-to-IRN crosswalk this repository does not hold",
which is why its fixture is 51 state rows rather than 14,106 systems. And the [census block
geography](census-block-geography.md) work needed it to place school districts inside House
districts. All 609 districts in the funding panel join through it with no losses.

**The general lesson, which this corpus has now recorded five times.** A blocker written down once
tends to be read afterwards as a fact about the source rather than as a note about an attempt. The
recorded blocker here was accurate about the identifier-change *series* and silent about
everything else in the file; nothing had tried the directory. It was one column, in a file the
corpus had already named, behind a URL that returns 200.

**What it does not support.** The consolidation history. A row here describes an agency as of
2022–23; districts that closed earlier are absent and districts renumbered since are not
reconcilable from this file alone. Any series spanning years still needs the derivation the
connector's original blocker describes.
