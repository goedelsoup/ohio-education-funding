# Chartered nonpublic enrollment by building, 1977–2025

**Source.** Ohio Department of Education and Workforce, **Enrollment Data**, under Frequently
Requested Data. Twenty-three attachments on one page: eighteen annual files, one an October, and
five compilations carrying the years before the annual series began.
**Type.** Primary source — the department's own October headcount of chartered nonpublic schools.
**Location.**
[Enrollment Data](https://education.ohio.gov/Topics/Data/Frequently-Requested-Data/Enrollment-Data),
every file a `getattachment` URL under that path. The filenames are inconsistent to the point of
being unusable as a pattern — `nonpub.xls`, `nonpub2015.xls`, `Revised-FY16-nonpub.xls`,
`web_nonpub_2011.xls`, `oct_08_fy09_adm_web_nonpub.xls` — so the twenty-three are **enumerated
from the page's anchors**, not composed. #257 recorded that this page reaches 1978 and no
connector owned it; this entry is the consequence.

**What it contains.** One row a chartered nonpublic school, or in the modern era one row a school
and grade: IRN, school name, and a count of pupils by grade and sex. The historical files add
county, school type and a parallel count by race. From October 2015 a separate sheet carries the
sector's true statewide total. Out-of-state pupils are counted separately throughout the modern
era.

**Why it matters.** R.C. 3317.024(E)(2)(d) divides the appropriation for auxiliary services
(R.C. 3317.06) and for nonpublic administrative cost reimbursement (R.C. 3317.062) by the
membership in chartered nonpublic schools **"as determined as of the last day of October of each
school year"**. This page is that determination. Before this connector the repository held the
denominator for one year — 711 schools and 173,156 pupils, from the 2023-24 Education Landscape
fact sheet — and every per-year question about the nonpublic aid lines was unanswerable. It is
also the only long series of the sector at all, which is what the scholarship participation
series has to be read against if participation is to mean anything.

**Every count under ten is masked, and it is not a small effect.** A cell printed `<10` stands
for some number from one to nine. Ohio's nonpublic sector is mostly small schools, and the
modern layout is one row a school and grade, so most cells in most schools are masked: in October
2023, 5,113 building-level cells are masked against a published in-state total of 171,830, and
the unmasked cells sum to 146,078. The building rows bound that October at **[146,078, 192,095]**
and no arithmetic over them recovers the published figure. Every quantity in the extract is
therefore a floor, a ceiling and a count of censored cells. Where a publisher states a true total
it is carried separately, in a column that says which publisher said it.

**The two totals a reader will reach for are different quantities.** This is the trap in the
source and it is worth stating twice.

- The historical compilations' `State Total` row is an Excel `SUM` over cells that had already
  been overwritten with the mask text, and `SUM` ignores text. It equals the sum of the unmasked
  cells exactly, on all twenty-eight sheets that carry one — the last three sheets of the
  2000-2008 compilation carry no such row. It is a floor wearing the label of a total:
  read as Ohio's nonpublic enrolment, 1977-78 comes out as 243,489 when the sector was between
  260,867 and 266,736.
- The annual files' state-totals sheet is a **true pre-masking total**, computed before the
  building rows were censored. October 2023's says 171,830 where its own building rows floor at
  146,078. From October 2015 every annual file carries one; before that, none does.

**Two blocks measure the same pupils and they do not agree.** The historical layout carries
enrolment twice — across twenty-six grade-and-sex columns and across five race columns. Same
population, so each bounds it, and the race block, being a fifth as wide, masks a fifth as many
cells and bounds it far better: 1977-78 is [243,489, 266,736] on grades and [260,867, 270,209] on
race. They are not merely differently precise. To 1982 a number of schools reported race and left
every grade cell zero, so the grade block is *absent* for them rather than masked; from 1983-84
the two agree on every building with no masked cell in either. Both are carried side by side and
neither is reconciled into the other. A caller wanting one number a building should prefer the
race block where it exists.

**`nonpub_fy17.xls` is October 2015's building data under October 2016's name.** Its building
sheet is `Revised-FY16-nonpub.xls`'s: 5,883 of 5,887 keyed cells are in both and **not one of the
shared cells differs**. Matched against the 2014-2019 compilation, which states a per-school
total for each of FY2014 through FY2019, the shared sheet agrees exactly with FY2016 on 86
unmasked schools and with no other year on more than six. So the rows are FY2016's, the
state-totals sheet beside them is genuinely FY2017's, and **October 2016 has no building-level
data of its own**. A pairwise sweep of the whole series found no other repost. Taking the sheet
at its filename publishes 2015's schools as 2016's, silently.

`Revised-FY16-nonpub.xls`'s own state-totals sheet is separately wrong: it reads 144,512, below
its own building rows' floor of 147,309, so it cannot be a total of them. The compilation's
171,395 is the figure for that October.

**Two publishers describe six Octobers and disagree.** The 2014-2019 compilation restates
FY2014–FY2019, which the annual files already cover, and the two do not line up. October 2017
reconciles exactly — 167,558 in-state plus 1,533 out-of-state is the compilation's 169,091. The
others do not: October 2016's annual total of 171,426 is the compilation's *total*, not its
in-state figure, and October 2018 reads 165,346 against the compilation's 165,511, which is
neither. The extract keys the sector panel on **October and basis**, so both publications get a
row and the disagreement is data rather than a resolution taken in a reader.

**The `_ADM` in the column names is not ADM.** Every annual file's notes sheet says the counts
are a headcount taken during the first full week of October and are *not* equivalent to average
daily membership — while the columns those counts sit in are named `BOYS_ADM`, `MALE_ADM` and so
on. The notes sheet is right.

**Layout eras.** Six, all resolvable by column name:

    1977-78 … 2006-07   one row a school; grades as Boys/Girls pairs, K through 12; race block
                        beside them. 2001-02 inserts a Total column and shifts the race block;
                        the sheets from 2002-03 are named "… ethnic data unavailable" and carry
                        no race block at all
    2007-08 …           one row a school AND grade. IRN, ORG_NM, grade, male, female, and from
                        2008 an out-of-state pair. Grade codes are "Kindergarten"/"1st Grade"
                        to October 2013 and "KG"/"01" after
    October 2009        building totals only: IRN, name, one number, 787 schools
    October 2015 …      a state-totals sheet joins the annual file, two rows wide
    FY2014 … FY2019     the compilation's By School sheet: FY, IRN, school, PS-12 total
    FY2014 … FY2019     the compilation's State Totals sheet, two grade spans an October

The sex columns are `BOYS_ADM`/`GIRLS_ADM` to October 2018 and `MALE_ADM`/`FEMALE_ADM` after;
three files spell the second one `FAMALE`. The preschool pair, where it exists, is school-level
and repeated identically on every one of a school's grade rows — summing it multiplies a school's
preschool count by its number of grades. No grade block in any era carries a preschool row, so
the annual files are K-12 and the compilation's per-school totals are PS-12.

**Status.** *Wired.* Forty-nine Octobers, 1977-78 through 2025-26, in
`crates/project/fixtures/nonpublic-building-panel.csv` and
`crates/project/fixtures/nonpublic-sector-panel.csv`. Both are built from one read of the
twenty-three files, so a failure skips both.

**What the extraction established, beyond the rows.**

- **The statutory denominator is the in-state count.** The FY2025 auxiliary services
  appropriation net of the College Credit Plus earmark, $164,077,754, over October 2024's 179,693
  in-state pupils is **$913.10**, and LSC publishes $913. Including the 1,478 out-of-state pupils
  gives $906. The per-pupil figure for the nonpublic aid lines is reproducible from this series
  and was not reproducible from anything the repository held before.
- **The single-year fact sheet reproduces exactly.** October 2023's state-totals sheet gives
  86,232 male and 85,598 female in state, 742 and 584 out of it — **173,156** — and its building
  sheet carries **711** distinct IRNs. Both are what the 2023-24 Education Landscape states, from
  a different office and a different publication.
- **711 was never short of anything.** The LSC redbook's 747 schools and greenbook's 725 read as
  a defect in the fact sheet's 711 until the series exists; on the series they are simply later
  Octobers — 711 in October 2023, 722 in October 2024, 749 in October 2025. The redbook was
  written against a vintage the fact sheet predates.
- **The sector's arc is not monotone.** It falls from a floor of 243,489 in 1977-78 to 176,811 in
  2006-07, with a local peak of 218,542 in 1996-97 on the way, and then falls again to 161,664 in
  October 2020. It has risen in each of the three Octobers since universal EdChoice eligibility:
  166,013, then 168,111, 171,830, 179,693 and 184,069.

**Other caveats:**

- **The district a building sits in is not published anywhere in this source.** Auxiliary
  services flows to a nonpublic school *through the district its building is located in*, so a
  per-district question about the nonpublic sector needs that column, and no file in any era
  carries it. County and school type exist for 1977-78 through 2006-07 and then stop. The
  Chartered Nonpublic School Information page publishes no directory, and
  `edchoice-designated-2627.csv` is public buildings only. The extract is keyed on building IRN
  alone for that reason, and the connector records the gap as still blocked rather than papering
  over it with a name match — [district names are not unique](../corpus/metric/enrolled-adm.yml)
  is a settled finding in this repository and school names are worse.
- **A building count is not a school count in the way a reader expects.** The panel counts
  distinct IRNs in a file, and an IRN is a chartered building: a school operating two buildings
  appears twice.
- **October 2009 contributes a bound and nothing else.** It is published as one number a school,
  so that year has no grade detail, no sex split and no out-of-state count.
- **October 2020 and October 2021 are pandemic Octobers.** The count moved for reasons the file
  does not record, and neither year has a second publisher to check it against.
- **The out-of-state columns begin in October 2008 and are masked like everything else.** They
  are small — 1,300 to 1,700 a year — and the masking is proportionally much heavier: October
  2023's out-of-state floor is under a tenth of its published 1,326.

## Used by

- [`crates/connect/src/fixtures/nonpublic.rs`](../../crates/connect/src/fixtures/nonpublic.rs) —
  the two panels and the masking arithmetic over them.
- [Auxiliary services](../corpus/program/auxiliary-services.yml), whose per-pupil figure and
  sector denominator are this series.
