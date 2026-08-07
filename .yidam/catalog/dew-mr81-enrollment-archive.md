# MR-81 October enrollment reports, 1998–2014

**Source.** Ohio Department of Education, MR-81 October enrollment report, one directory per
year.
**Type.** Primary source — the department's own annual enrollment report.
**Location.** `https://public.education.ohio.gov/MR81/`, an open directory listing. Seventeen
year directories, `MR81_October_1998` through `MR81_October_2014`, each holding a delimited text
file, a fixed-width text file, a ReadMe, and a zip.

**What it contains.** October enrollment by district, as reported on the MR-81 form. Seventeen
consecutive Octobers.

**Why it matters here.** This corpus has **three** enrollment observations — FY2024, FY2025 and
FY2026 — and that shortage is load-bearing rather than cosmetic. It is why a projection interval
here rests on the *cross-sectional* spread of district growth rates rather than on each
district's own variability: three points can fit a trend and cannot say how wrong it might be.
See [enrolled ADM](../corpus/metric/enrolled-adm.yml), where the limitation is recorded, and
`project::series`, where the prior it forces is implemented.

Seventeen years of district enrollment would let that interval come from the thing it is
supposed to describe.

**Status.** *Retrievable, unparsed.* Found while searching for the voucher deduction channel,
which is not here. Recorded rather than acted on, because taking it up properly is a phase of its
own and carries the caveats below.

**Caveats:**

- **This is headcount, not ADM.** MR-81 is a count on one October day; average daily membership
  is an average over the year, and the Fair School Funding Plan funds on ADM. The two move
  together and are not the same number, and
  [enrolled ADM](../corpus/metric/enrolled-adm.yml) already records that they must not be
  substituted for one another. A variability estimate drawn from headcount growth and applied to
  an ADM projection is an **inference**, and would have to be tagged as one.
- **It stops in 2014.** There is a nine-year gap between the end of this archive and the FY2024
  start of the calculator's ADM series. Whatever fills it is not in this directory.
- **District identity is not stable across seventeen years.** Consolidations, closures, and IRN
  changes mean a naive join produces a panel whose membership silently varies — the exact
  failure the `nces-ccd` connector was approved to handle and which remains unbuilt. A long
  series assembled without it is wrong in a way that looks like a trend.
- **Layout is per-era.** Both a delimited and a fixed-width file are published per year and the
  ReadMe differs between directories. The same per-era reader problem as
  [`census-f33`](../../crates/connect/sources/census-f33.md) and as the pre-2022 five-year
  forecast filings.
