# Common Core of Data: local education agency directory

**Source.** National Center for Education Statistics, U.S. Department of Education.
The LEA universe directory, thirty school years of it: 1994–95 through 2023–24.
**Type.** Primary source — the federal register of every public school district in the country,
compiled from state education agency reporting.
**Location.** `nces.ed.gov/ccd/data/zip/`.

**What it contains.** One row per local education agency, 1,058 of them in Ohio in 2022–23: the
federal `LEAID`, the state's own identifier in `ST_LEAID`, name, type, operational status, and
address. Ohio has run between 781 agencies a year (1996–97, before it had a community school) and
1,231 (2005–06, at the top of the charter opening wave).

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

**Thirty years of it are now held**, school years 1994–95 through 2023–24, one row per Ohio
agency per year: `crates/dispersion/fixtures/ccd-lea-directory.csv`, 30,655 agency-years.

**Five physical eras, and the seams are where the join breaks.**

    1994-95 .. 1997-98   fixed-width, no header, 1,030-byte records, 30-character names
    1998-99 .. 2006-07   fixed-width, no header, record 722 → 530 over nine years
    2007-08 .. 2013-14   tab-delimited, header present, columns year-suffixed to 2009-10
                         (STID08, BOUND08, TYPE08 → STID, BOUND, TYPE)
    2014-15              still tab, but the state-id column is renamed ST_LEAID
    2015-16 .. 2023-24   comma-delimited; from 2016-17 ST_LEAID gains the `OH-` prefix

The state identifier is the Ohio IRN in every year, six digits zero-padded. It is written bare
through 2015–16 and as `OH-043786` from 2016–17. A reader that assumes either form silently
matches nothing for half the window.

**The file names are keyed on the year the survey closed, not the year it describes.** `pau94datr`
is 1994–95 and `pau95data` is 1995–96; `ag981c` is 1998–99 and `ag081a` is 2008–09. This repository
had the convention recorded one year later than it runs, and the record lengths it had recorded
alongside — 722 for the first of the nine middle years, 530 for the last — belong to the correct
alignment and were what caught it. Each file's own layout document names its school year twice,
which is the check.

**In the fixed-width era almost nothing this reader needs moves.** The agency identifier is bytes
1–7 in every one of the thirteen years, the state identifier 8–21, and the name starts at 22. The
name's end and the agency type move once, together, in 1998–99. The status byte moves four times:
162 → 280 → 281 → 284 → 309. The *record* moves in seven of the nine years between 1998–99 and
2006–07 and almost none of that is in the first 300 bytes.

**Ohio is selected on `FIPST`, never on `LSTATE`.** They disagree, in both directions. LEAID
3901497, Urban Pathways of Youngstown, is filed `FIPST=39` with `LSTATE=PA` in 2012–13 and 2013–14
— a mailing address rather than a jurisdiction. Earlier years file an Arizona agency as
`LSTATE=OH`, with an Ohio-shaped six-digit state id beside it.

**What it does not support, and the reason is stronger than "not recorded".** The consolidation
history. The CCD defines eight operational-status codes and exactly one marks a consolidation:
code **5**, *"significant change in geographic boundaries or instructional responsibility"*.
**Ohio has never filed it** — zero occurrences in 30,655 agency-years, which is every Ohio
agency-year the directory has published since 1994–95.

What Ohio files instead, for **all 689 departures without one exception**, is code **2**: *"closed
with no effect on another agency's boundaries."* For Bettsville Local, Ledgemont Local and Newbury
Local — whose territory went to Old Fort, Berkshire and West Geauga respectively — that statement
is false. And the receiving districts' rows do not change: all three are coded `1 Open` in the year
before, the year of, and the year after.

So the source does not merely omit the answer. It asserts the wrong one, about the departing
agency, while leaving the surviving agency unmarked. Every derivation from that was tested and
fails: enrolment absorption on the survivor is confounded roughly fifty to one by ordinary growth;
name changes on the survivor arrive up to two years out of alignment and share a vocabulary with
cosmetic re-spellings. Settling the reason needs Ohio's own territory-transfer orders under
R.C. 3311.22, which are resolutions in an educational service center's minute book — not published,
but recited by the Auditor of State on the *receiving* body. See
[auditor-district-audits](auditor-district-audits.md).

**What the departures actually are.** 689 across thirty years: **616 community schools**, 66
service agencies, five regular districts, one local district and one state agency. Two of the five
regular districts are STEM schools, which Ohio funds as their own unit under R.C. 3326 and the
federal directory types as districts.

**And the arrival code says the same thing.** Ohio forms two school districts in thirty years —
Monroe Local in 2000–01 and Manchester Local in 2004–05 — and files both under code **3**, *"a new
education agency formed with no effect on another agency's boundaries."* Middletown **Monroe** City
is renamed Middletown City in the same year Monroe Local appears, same identifier, coded open on
both sides; Adams County/Ohio Valley Local does not change at all in the year Manchester Local
appears. Both new districts carry IRNs from Ohio's original county-ordered block. A third,
`PEEBLES`, is filed in 2004–05 as *"scheduled to be operational within 2 years"* and closed the
next year without ever reaching open.

**Three of the years recode the agency type without an agency moving**, 532 agencies in all: 47
from type 1 to type 7 in 2000–01 (Ohio's first community schools, filed as school districts for
two years), 60 from type 3 and 49 from type 1 into type 4 in 2002–03, and 376 from type 2 into
type 1 in 2006–07. Counting Ohio's districts as types 1 and 2 gives 661, then 675 and 708, then
662, then 613–622 — three discontinuities, none of them a district opening or closing.

**The 66 are what the fourteen older years added.** Ohio did not have educational service centers
in 1994–95; it had **86 county boards of education**, and this file watches thirty-nine of them
disappear between 1995–96 and 2001–02 while multi-county centres appear beside them — Athens and
Meigs leave, Athens-Meigs arrives; Ross and Pike leave, Ross-Pike arrives. Fourteen such centres
join, so the category holds 61 by 2001–02; sixty are recoded from type 3 to type 4 in 2002–03 and
the last leaves. Every one of the forty is filed under code 2.

That matters because R.C. 3311.22 vests a territory transfer in an **educational service center
governing board**. The register denies the consolidation of the bodies whose minute books hold the
orders it cannot describe — and it does so a generation before the district cases, which settles
that this is the source's standing practice and not a lapse in the 2010s.

The county-to-compound-name correspondence is legible and is **not a filed fact**. It is a reading
of a name field, it is up to two years out of alignment, and the same field moves one survivor
between "Guernsey-Monroe-Noble" and "Ohio Valley" and back across four years. Twice, in 1994–95
only, a clerk wrote the answer into the name outright — `ASHLAND (SEE ASHLAND-WAYNE) C` and
`WYANDOT (SEE SENECA-WYANDOT)` — and never again.

**The last of the 66 is the one with an instrument behind it.** Geauga County ESC's final audit
recites its merger with Lake County ESC into the Educational Service Center of the Western Reserve
on 7 November 2019. The directory files Geauga County ESC closed with no effect on anyone; Lake
County ESC keeps `LEAID 3904786` and IRN 047860, is coded `1 Open` throughout, and takes the new
name in 2020–21. See [auditor-district-audits](auditor-district-audits.md).

**Nine older years exist and are not held.** The survey reaches back to 1986–87 in the same
fixed-width family (`pau86data.zip` onward). They are left out because nothing consumes them and
because the negative above is already made over a completely enumerated thirty years.

**Every published revision was compared where more than one is served.** NCES links only the latest
(`ag981c`, `ag991b`, `ag031b`, `ag041c`, `ag061c`, `ag071b`) and keeps the earlier ones reachable.
For 1999–2000, 2003–04, 2004–05, 2006–07 and 2007–08 the superseded edition was fetched and its
Ohio rows are identical to the wired one **in the five columns this reader takes** — identifier,
state identifier, name, type, status. That is not a claim that the files are identical.
