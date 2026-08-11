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

**FY2022 is the peak of federal pandemic relief.** The federal share is inflated and the state
and local shares correspondingly deflated. Every finding above therefore understates Ohio's local
reliance rather than overstating it, which is the safe direction.

**And the series across years now exists, with the relief years marked.** Ten years of the survey
are held for Ohio — FY2012, FY2013 and FY2015 through FY2022 — in
[`f33-ohio-panel.csv`](../../crates/dispersion/fixtures/f33-ohio-panel.csv), read by
`dispersion::ohio_panel`. Two federal spikes fall inside the window and both must be marked
before any point is compared to any other: the ARRA tail, which puts FY2012 at an 8.95% federal
share against FY2013's 5.25%, and ESSER, which puts FY2022 at 14.02%.

**FY2014 is missing from the archive.** `sdf14_1a.zip`, `sdf141a.zip`, `sdf14_2a.zip` and
`sdf14_1a_rev.zip` all return 404 while FY2013 and FY2015 answer under two of those patterns. Nine
intervals across ten years and one of them is two years wide. Nothing interpolates it.

**The layout is per-era and only the names survive.** The three eras carry **256, 260 and 354
columns**, and the archive member is `sdf121a.txt` in one year and `Sdf16_1a.txt` in another — so
the reader resolves every column by header and matches the member by extension. A positional map
written against FY2022 would read the wrong field in FY2012 and report it as a number.

## The per-district view, and the filter it needed

**Update.** The corpus now holds this survey per district as well as per state, from
`sdf22_1a.zip` — NCES publishing the same collection keyed on `LEAID` rather than the Bureau's
`IDCENSUS`. That key is what made the join possible: `LEAID` reaches Ohio's IRN through the [CCD
directory](nces-ccd-lea-directory.md), and all 609 districts in the funding panel come through with
no losses.

**The comparison set is not every agency, and getting that wrong is easy.** The survey's unit is a
local education agency, and 357 of Ohio's 968 rows are community schools, joint vocational
districts and educational service centres. A community school raises almost no local tax by
construction, so leaving them in drags the distribution somewhere no traditional district lives —
the first attempt here put Ohio's 200 smallest agencies at an **8% local share**, which is a true
fact about charter finance and a useless one about school districts.

The filter is the survey's own: `AGCHRT != 1` and `SCHLEV == 03`, leaving **10,382 unified,
non-charter districts nationally**. It costs one Ohio district, a K-8 agency carried without a
national position rather than given an invented one.

**The two files agree, which is the check that the filter is right.** This catalog entry records
Ohio at **51.8%** local share from the Bureau's state-level table. The district file, filtered and
re-aggregated, gives **51.7%** — two independently assembled sources within a tenth of a point. A
filter that admitted Ohio's community schools would not reproduce it.

**And the district view says something the state view cannot.** Ohio's *median district* sits at
the **66th national percentile** on local share, and its quarter-poorest at the national median.
The shift between the Ohio and national views is largest in the *middle* of the distribution and
smallest at the tails — the opposite of the intuition that a national comparison mainly relocates
extremes.

## The three archives that were not where the others are

The Ohio panel opened at FY2012 for a year and a half because `sdf10_1a.zip`, `sdf101a.zip` and
their obvious variants all 404. FY2009 through FY2011 are published under a **`_txt` suffix** the
FY2012-FY2022 files do not use — `sdf091a_txt.zip`, `sdf101a_txt.zip`, `sdf11_1a_txt.zip` — and
answer 200. All three carry the same 256 tab-delimited columns and the same names the builder
already looks up by header, so nothing but the registry entries had to change. [verified]

That mattered more than a tidier series. FY2012 is **three years inside** the FY2010-FY2014 real
spending trough, so the panel had been unable to see the thing it was best placed to measure, and
two findings were sitting behind a filename:

- The state share of district revenue falls **11.7 points** from FY2009 (45.9%) to FY2022
  (34.2%). From FY2012 it falls 7.7. Both are correct about their own window.
- FY2010 and FY2011 are the only years on record where state aid closes about a **third** of the
  local revenue gap rather than its usual ~45%, while federal gap-closing roughly doubles. FY2009
  brackets them from before at 47.2% and FY2013 from after at 44.3%.

**FY2014 is genuinely absent**, not misnamed: `sdf14_1a.zip`, `sdf141a.zip`, `sdf14_2a.zip`,
`sdf14_1a_rev.zip` and the `_txt` forms all 404 while FY2013 and FY2015 answer under two of those
patterns. The panel states the gap rather than interpolating across it. [open]

## The FY2024 file is a different shape at the same URL

`elsec24t.xlsx` — `.xlsx` where FY2022 is `.xls` — drops the `IDCENSUS` column, which shifts
every later index by one. Every column the corpus wants still exists and still means the same
thing, so a positional read would have produced a complete, plausible, entirely wrong extract:
state codes read as unit types, revenue read as enrolment. `build_f33_states` was positional and
is now header-driven; it reproduces the FY2022 fixture byte for byte, which is the check that the
change was safe. [verified]

Two naming traps worth stating, because both have cost time here:

- These are the Bureau's `elsec` tables. The NCES `sdf` school-district files the Ohio panel reads
  are a different product with different names — property tax is `LOCRPROP` here and `T06` there.
- FY2023 and FY2024 exist only as `.xlsx`; the `.xls` URLs time out rather than 404, which reads
  as a network problem rather than a missing file.

**And the published state per-pupil figures are not reconstructible from the unit rows.** Summing
`TCURSPND` over `ENROLL` gives errors that scatter by state — Indiana +0.5%, Ohio −3.7%, Michigan
−9.8% against the FY2024 published table — under every spending definition and unit filter tried.
The revenue *shares* reproduce within a point. Use this file for mix, not for level. [open]
