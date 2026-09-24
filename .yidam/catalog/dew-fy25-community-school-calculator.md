# FY25 Community/STEM School State Foundation Funding Simulator

**Source.** Ohio Department of Education and Workforce, Office of Budget and School Funding.
`FY25-CS-State-Foundation-Funding-Calculator-3-7-2024-1.xlsx`, last modified 8 March 2024 by its
own `docProps` — the filename's date and the file's agree.
**Type.** Primary source — the department's own working model, not a report about it.
**Location.** **Archived.** `web.archive.org/web/20250222101322id_/`, over
`education.ohio.gov`, Finance and Funding → School Payment Reports → State Funding for Schools →
Community School Funding. The department serves only the current year's calculator at that
address and replaces it in place; the FY2025 file has no live URL. The admissibility rule is
[`an-archived-source-is-still-a-source`](../decisions/an-archived-source-is-still-a-source.yml),
and the condition that matters — that the bytes prove they are the publisher's by reproducing the
publisher's own arithmetic — is met below.

**What it contains.** Twenty-five worksheets, **21 of them hidden**, computing FY2025 state
foundation funding for
**343 community and STEM schools**: base cost, special education, DPIA, English learner and
career-technical by category, core foundation funding, transportation, the equity supplement, the
formula transition supplement, and facilities. Plus enrolled ADM, the FY2020 and FY2021 funding
bases the transition supplement is computed against, and a school list.

The sibling of [`dew-fy27-community-school-calculator`](dew-fy27-community-school-calculator.md),
two years earlier, and the second of the two per-school formula sources this corpus holds.

**Why it was retrieved: the rate, which is the thing that moved.** R.C. 3317.022 as H.B. 96
codified it runs $650 in FY2025, $500 in FY2026 and $400 in FY2027. Only the last had a
department model behind it; the other two were carried from the enacted act and the greenbook,
which state all three in one sentence each. This file is the department paying the first of them.

The workbook writes the formula out on its `Detailed SFPR ` display sheet, row 68 —
**`Equity Supplement [a*$650]`**, where `a` is enrolled ADM — and the arithmetic closes on every
row: **317 site-based schools paid $650 × enrolled ADM with zero mismatches and exactly one
distinct per-pupil ratio**, `650.000000`. The **8 STEM schools and 18 e-schools are paid exactly
zero**, which is the same eligibility rule the FY2027 file carries and the same one the
[redbook](lsc-dew-redbook.md) says the *introduced* FY2026-27 budget would have changed. Two
years apart, under two rates, the population rule did not move.

**How the bytes prove they are the department's.** The archived file reproduces the department's
own arithmetic three ways, none of which a corrupted or substituted file would survive:

- `Detail SFPR` and `Summary_SFPR` compute the supplement separately and **agree on all 343
  schools**, to the cent.
- The `State of Ohio` aggregate row, IRN `050765`, reads **$54,846,889.8614** and the 343 schools
  beneath it sum to that figure exactly.
- The display sheet's own statewide cell carries the same $54,846,889.86 against the bracketed
  `[a*$650]`.

The capture is whole: 1,347,923 bytes, a clean zip, SHA-256
`785d40413337d4cf4c1dd31adc2b69b8d31c92e23c39819154b57cc1ec1b50ac`, pinned in
`crates/connect/source-digests.txt`. That is worth saying because the FY2025 **traditional**
calculator — a different file, one directory across — is *not* whole: its captures are truncated
at a megabyte, and the decision record cites it as the example of an archived file that stays
unrecoverable. Two FY2025 calculators, two outcomes; the difference is what the crawler got, not
what the rule permits.

A seventeen-column extract is committed at
[`crates/dispersion/fixtures/fy25-community-school-funding.csv`](../../crates/dispersion/fixtures/fy25-community-school-funding.csv),
built by the same
[`connect::fixtures::community_schools`](../../crates/connect/src/fixtures/community_schools.rs)
that reads the FY2027 file, which refuses either workbook if a site-based school's payment is not
rate times ADM or if an e-school or STEM school is paid anything at all.

**Access constraints.** 1,347,923 bytes XLSX, no credentials, but reachable only through the
Internet Archive. Cached formula results are present, so values read without a spreadsheet
engine. Individual sheets carry a protection flag and there is no workbook structure lock;
neither impedes reading.

**Caveats.**

- **It is an actual year read from a projection's file.** FY2025 has happened, but this is the
  simulator the department published in March 2024 — its enrolled ADM is projected, and the
  payments in it are what the formula was expected to produce, not what was finally paid. The
  final FY2025 payment report is a separate document. What the file establishes is the **rate**,
  which is a parameter of the formula rather than an outcome of it, and the rate is the same
  number in a projection and an actual.

- **The layout is not the FY2027 layout, in four places.** The summary sheet's calculated base
  cost is `Ab. Base Cost (new)` where FY2027 writes `A. Base Cost Calculated`, and the same `b`
  suffix runs across special education, DPIA, English learner and career-technical; the column
  letters run one behind, so total state support is `K. Total State Supports` where FY2027 says
  `L.`; the equity supplement sits at the far right of the detail sheet rather than in the middle
  of it; and the `E-School Designation` column spells a STEM school **`S`**, not `STEM`. The
  extractor reads a named vintage rather than sniffing, and refuses each year's file under the
  other year's spellings.

- **There is no base funding supplement, because H.B. 96 had not created it.** The FY2025 file has
  no such column. The committed extract leaves that cell **empty rather than zero**: a school paid
  nothing and a payment that did not exist are different facts, and only the second is true here.

- **The detail sheet has two total columns and the first one is not the total.**
  `J. Total State Supports` is total state support *before* the equity supplement;
  `K Total State Support [F+G+H+I+J]` is the one that includes it. Reading by heading prefix gets
  a figure short by exactly the line this file was retrieved for.

- **Two years is not a series.** FY2025 and FY2027 are two years apart over a population that
  opened and closed schools in between, under a phase-in that moved from 0.6667 to 1, and with a
  funding line created between them. Differencing them measures four changes at once.

- **No local share exists to be missing.** Community and STEM schools have no taxing authority,
  so the state share is 100% and the valuation, income and capacity columns the district model
  turns on are absent by construction rather than unpublished.

## Used by

- [`formula-component/fsfp-community-school-equity-supplement`](../corpus/formula-component/fsfp-community-school-equity-supplement.yml)

## Feeds connector

[`dew-foundation`](../../crates/connect/sources/dew-foundation.md)
