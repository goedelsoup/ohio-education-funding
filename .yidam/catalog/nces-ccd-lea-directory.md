# Common Core of Data: local education agency directory

**Source.** National Center for Education Statistics, U.S. Department of Education.
The LEA universe directory, sixteen school years of it: 2008–09 through 2023–24.
**Type.** Primary source — the federal register of every public school district in the country,
compiled from state education agency reporting.
**Location.** `nces.ed.gov/ccd/data/zip/`.

**What it contains.** One row per local education agency, 1,058 of them in Ohio in 2022–23: the
federal `LEAID`, the state's own identifier in `ST_LEAID`, name, type, operational status, and
address. Ohio has run between 1,047 and 1,176 agencies a year across the sixteen years held.

**Why this file, out of all of CCD.** The corpus named `nces-ccd` as a connector since genesis and
left it `Declared`, blocked on the fact that a long identifier-change series — which districts
merged, split, or were renumbered across decades — has to be derived rather than read.

Half of that has since been answered and half has been answered in the negative. *Which* agencies
existed in which year is read straight off the file, once you hold more than one year of it. *Why*
any of them stopped is not derivable at all, and the reason is not that the field is empty — see
below.

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

**Sixteen years of it are now held**, school years 2008–09 through 2023–24, one row per Ohio
agency per year: `crates/dispersion/fixtures/ccd-lea-directory.csv`, 17,618 agency-years.

**Three physical eras, and the seams are where the join breaks.**

    2008-09 .. 2013-14   tab-delimited, header present, columns year-suffixed to 2009-10
                         (STID08, BOUND08, TYPE08 → STID, BOUND, TYPE)
    2014-15              still tab, but the state-id column is renamed ST_LEAID
    2015-16 .. 2023-24   comma-delimited; from 2016-17 ST_LEAID gains the `OH-` prefix

The state identifier is the Ohio IRN in every year, six digits zero-padded. It is written bare
through 2015–16 and as `OH-043786` from 2016–17. A reader that assumes either form silently
matches nothing for half the window.

**Ohio is selected on `FIPST`, never on `LSTATE`.** They disagree, in both directions. LEAID
3901497, Urban Pathways of Youngstown, is filed `FIPST=39` with `LSTATE=PA` in 2012–13 and 2013–14
— a mailing address rather than a jurisdiction. Earlier years file an Arizona agency as
`LSTATE=OH`, with an Ohio-shaped six-digit state id beside it.

**What it does not support, and the reason is stronger than "not recorded".** The consolidation
history. The CCD defines eight operational-status codes and exactly one marks a consolidation:
code **5**, *"significant change in geographic boundaries or instructional responsibility"*.
**Ohio has never filed it** — zero occurrences in 17,618 agency-years here, and zero in the
twenty-six years back to 1998–99 that were checked and not wired.

What Ohio files instead, for **all 341 departures without one exception**, is code **2**: *"closed
with no effect on another agency's boundaries."* For Bettsville Local, Ledgemont Local and Newbury
Local — whose territory went to Old Fort, Berkshire, and West Geauga and Chardon respectively —
that statement is false. And the receiving districts' rows do not change: all four are coded
`1 Open` in the year before, the year of, and the year after.

So the source does not merely omit the answer. It asserts the wrong one, about the departing
agency, while leaving the surviving agency unmarked. Every derivation from that was tested and
fails: enrolment absorption on the survivor is confounded roughly fifty to one by ordinary growth
and gets the sign wrong on Newbury, where Chardon *lost* pupils the year it received them; name
changes on the survivor arrive up to two years out of alignment and share a vocabulary with
cosmetic re-spellings. Settling the reason needs Ohio's own territory-transfer orders under
R.C. 3311.

**What the departures actually are.** 341 across the window: **327 community schools**, nine
service agencies, five regular districts — and two of those five are STEM schools, which Ohio
funds as their own unit under R.C. 3326 and the federal directory types as districts.

**The fourteen earlier years are retrievable and unwired.** School years 1994–95 through 2007–08
all return 200 to this project's user agent. They are a second reader: fixed-width with no header,
and the column positions move in seven of the nine years between 1998–99 and 2006–07.
