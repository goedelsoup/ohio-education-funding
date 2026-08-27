# MR-81 free and reduced-price lunch reports, 1998–2025

**Source.** Ohio Department of Education, **Office for Child Nutrition**, "LUNCH MR 81 Report for
October", one directory per year.
**Type.** Primary source — the department's own annual child-nutrition report.
**Location.** Two, and the second is the one this entry used to deny existed.
`https://public.education.ohio.gov/MR81/` is an open directory listing of seventeen year
directories, `MR81_October_1998` through `MR81_October_2014`. It really does stop there — the
Internet Archive holds 101 URLs under it and the newest is October 2014.

**The report did not stop; the directory did.** From October 2015 it is an attachment on
[Data for Free and Reduced Price Meal Eligibility](https://education.ohio.gov/Topics/Student-Supports/Food-and-Nutrition/Resources-and-Tools-for-Food-and-Nutrition/Data-for-Free-and-Reduced-Price-Meal-Eligibility),
one `.xlsx` an October, and that page links back to the open directory for "years prior" — the
department treats them as one series. Two of the eleven paths still carry `MR81` in them. Three
different parent segments and three filename conventions, one of them misspelling `Eligibilty`, so
the eleven are scraped off that page rather than composed from a pattern. See #16.

**THIS ENTRY DESCRIBED THE WRONG SOURCE.** It was written from the directory listing and said
MR-81 holds "October enrollment by district, as reported on the MR-81 form", and the corpus
planned a phase around it as an enrollment archive. Opening a file says otherwise: MR-81 is the
**free and reduced-price lunch report**, published by the Office for Child Nutrition, **one row
per school site** grouped by sponsor. It carries an enrollment column because a lunch claim needs
a denominator, not because it is an enrollment report.

Everything below is from the files. The slug is left alone because it is cited, and renaming it
would break the citation to make the mistake less visible.

**What it contains.** Per school site: sponsor, county, site IRN, kitchen type, an enrollment
count, free lunch applications, reduced-price lunch applications, their total, and both as
percentages of enrollment. Sponsor subtotals in the rendered files.

**Why it is worth more than the enrollment framing suggested.** Seventeen consecutive Octobers of
free and reduced-price counts is a **district poverty series**, and the corpus has none. Ohio's
[disadvantaged pupil impact aid](../corpus/formula-component/fsfp-disadvantaged-pupil-impact-aid.yml)
runs on an "economically disadvantaged" count whose definition R.C. 3317.03(B)(21) hands to the
department, and free-lunch eligibility has historically been the department's operative test. A
seventeen-year run of it is the closest thing available to a long series of the count DPIA is
paid on.

**Four breaks, each of which produces a wrong series if unhandled:**

- **The enrollment column changes definition in 2010.** `AdmCount` and `PctFreeAdm` become
  `CECount` and `PctFreeCE`. The 2014 header states what CE means: *"Current Enrollment (CE) —
  highest daily number of students with access to the program."* That is not average daily
  membership and not the same quantity as what preceded it. A reader resolving columns by
  position would splice the two silently.
- **The poorest sponsors leave the main file in 2012.** From FY2012 the report splits into
  **Traditional, Provision 2, and Community Eligibility (CEP/CEO)** streams — in 2012 as three
  subdirectories, in 2013 and 2014 as separate files. The 2014 Traditional header says outright
  that it *"Excludes Provision 2 and Community Eligibility Option (CEO) sponsors."* Districts
  adopt CEP precisely because their poverty is high enough to qualify for universal free meals,
  so a series built from Traditional alone drops the highest-poverty districts, and drops more of
  them every year as adoption spreads. This is the single most dangerous property of the source.
- **Sponsors are not districts.** The 2005 file's first rows are a Residential Child Care
  Institution and a county Board of MR&DD. Any district panel needs a sponsor-type filter.
- **The 1998-2000 files have no sponsor-type column at all**, so there is nothing to filter on.
  They are one row per school with the district named and numbered on it, and the type has to come
  from 2001, which leaves some thirty-five sponsors a year — three thousand to eight thousand
  pupils, against 1.8 million — with no type anything states.

**Layout eras.** Four, all resolvable by column name:

    1998         comma, school-centric: BREAKFAST IND, SCHOOL, SCHOOL IRN, ..., DISTRICT IRN
    1999-2000    comma, same columns, published as a "Delimited" file
    2001         comma, sponsor-centric: County, SponsorIRN, ..., AdmCount, ...
    2002-2009    tab, sponsor-centric; ProvisionYear appears in 2003
    2010-2011    tab, sponsor-centric, AdmCount renamed CECount
    2012-2014    three programme streams, separately published
    2015-2025    one workbook an October, streams on a column, counts masked under ten

Every year from 1999 carries a delimited file alongside the rendered text; 1998 is comma-delimited
already. File naming is inconsistent to the point of needing a per-year table —
`MR81-Oct2002-TabDelimited.txt`, `MR81_Oct_2010_delimited Revised 0911.txt`,
`MR8_Oct_2011_Delimited.txt` — and one year's name is misspelled.

**Status.** *Wired in full.* **Twenty-eight Octobers** in thirty-two files, across four readers: a
fixed-width one for 1998–2000, a delimited one for 2001–2014, one for the printed report — the only
form the 2013 community-eligibility stream was ever posted in — and a workbook reader for
2015–2025. The panel is `crates/dispersion/fixtures/mr81-sponsor-panel.csv`, 33,281 rows.

**Two things the workbook era carries that the delimited one did not.** Its Notes sheet *states*
the community-eligibility rule this repository had previously measured off the 2014 file — the
printed percentage is the directly-certified count times 1.6 over enrolment — so the ceiling half
of the corpus's band is the programme's own arithmetic rather than an inference. And it **masks a
count printed as `<10`**, which is not a bound: when either of the free/reduced pair falls under
ten *both* are masked, so the value behind one can be 15 or 19. The masked count is recovered from
the percentage printed beside it, on an identity that reproduces all **42,983** unmasked cells
exactly. `censored` on each row says how many arrived that way.

**What the extraction established, beyond the rows.**

- **The 1998–2000 files are fixed-width, not comma-separated.** Every row is exactly 170 bytes
  with the thirteen separators at the same offsets, so the seven district names a year carrying a
  comma never shift anything. The earlier reading of this entry — that they need "a last-field
  rule" — was describing a problem that only exists if the file is split on its delimiter.
- **They were written in 2004, not in their own years.** All three carry the same 4,246 schools,
  zero-filled where a school was not participating, and the directory dates them 5/2004 and
  8/2004. October 2000 is the one year whose contemporaneous printing also survives, and the two
  agree on 4,233 of 4,236 schools — which is what makes the back-fill usable.
- **2001's delimited file was being misread, and the corpus had published the result.** It is the
  only comma-delimited year; nine rows carry a comma inside a school name; two of them are
  Cleveland City SD, and read positionally they put site IRNs `00026450` and `00093153` in the
  enrollment column. The statewide public figure was 1,921,522 against a true 1,802,937 and the
  poverty share was 27.7% against 29.5%. Repaired by anchoring the row's nine trailing fields from
  the right; after the repair, free and reduced approvals match the printed report **exactly for
  all 1,227 sponsor blocks**.
- **The three streams count three different things.** Traditional is current-year applications.
  Provision 2 is applications frozen at the base year the file names — the same sponsor reports
  the same counts in 2012, 2013 and 2014. Community eligibility collects **no applications at
  all**; its `PctFreeRedCE` is the directly-certified count times 1.6 capped at enrollment, which
  reproduces in all 735 of the 2014 rows. Joining Traditional alone is the trap this entry always
  warned about; joining all three by their approval columns is a *different* trap with the same
  shape, and both make poverty appear to collapse.
- **2013's community stream has no delimited file.** Only the printed report, which names its
  sponsors and neither numbers nor types them, so the identifiers come from the delimited files
  either side. `Believe to Achieve-Canton` is filed under Cuyahoga in 2012 and Stark in 2013, so
  the lookup falls back from name-and-county to name.
- **The 2012 community file's header is one column wider than its rows.** A blank column name sits
  before `CEOEligibleStudents`, so resolving that column by name points one past the end of the
  data — silently, since the trailing columns are all numeric.

**Other caveats:**

- **This is a count of applications, not of eligible children**, in the years before direct
  certification. Under-application among eligible families is real and varies by district.
- **Two Octobers are not a reading of Ohio.** Under USDA's nationwide free-meal waivers, in force
  for 2020-21 and 2021-22, a sponsor could serve every student free without collecting one
  application — and almost every one stopped filing. October 2020 and October 2021 carry **296 and
  261 sponsors against about 850**, and a quarter of the enrolment; their band reads twenty points
  above the years either side and every point of that is about who filed. The feed marks them
  `comparable: false` and the page prints "not the state" in the row.
- **Traditional is emptying into community eligibility.** 2,849 sites in October 2015 against
  **1,782** in October 2025, while CEP goes 844 to 1,754. So no site count can serve as a floor on
  a parse in this era, and the extractor floors the October instead.
- **District identity is not stable across twenty-eight years.** Consolidations, closures and IRN
  changes mean a naive join produces a panel whose membership silently varies — the same problem
  `dispersion::ohio_panel` now measures rather than assumes for the F-33, where the FY2022-23
  directory names 124 fewer FY2012 agencies than FY2022 ones.

## Used by

- [`crates/dispersion/src/mr81.rs`](../../crates/dispersion/src/mr81.rs) — the panel and the
  aggregates over it, including the band the split years carry instead of a share.
- The `meal_program` block of the feed, which is the card "What the poverty weight is
  counted on" on the site's `/history` route.
- [DPIA](../corpus/formula-component/fsfp-disadvantaged-pupil-impact-aid.yml), whose fourteen-year
  poverty series is this file.

The [enrolled ADM](../corpus/metric/enrolled-adm.yml) node records the three-observation
limitation this source was expected to relieve; on the corrected reading it would relieve it only
as an inference, since CE and headcount are neither of them ADM.
