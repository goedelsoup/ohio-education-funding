---
used-by:
  - ../corpus/program/scholarship-donation-credit.yml
---
# Tax Expenditure Report

**Source.** Ohio Department of Taxation, Tax Analysis Division, published with the executive
budget as part of *Book Two*. Two editions are held: the FY2024-2025 report, from the archived
FY2024-25 executive budget, and the FY2026-2027 report.
**Type.** Secondary source — the department's own estimates of revenue forgone, not a record of
returns.
**Location.** The FY2024-25 edition is `Book_Two_Tax_Expenditure_Report.pdf` on
`archives.obm.ohio.gov`; the FY2026-27 edition is `Tax_Expenditure_Report_2026-2027_-_Final.pdf`
on `dam.assets.ohio.gov`. Both are digest-pinned by the `tax-income-credits` connector.

**What it contains.** One entry for each tax expenditure, with its statute, the year it was
enacted, a one-paragraph description, an estimate for each of four fiscal years in millions of
dollars, and a data source code. Code A is the department's own return data, B is data from other
governments, and C is everything else.

**What this corpus reads.** Entry 2.25, the credit for donations to scholarship organizations,
R.C. 5747.73. The fixture `sgo-credit-estimates.csv` holds both editions' four years and their
codes. The reader requires exactly one line beginning `R.C. 5747.73;` and takes the `Estimate:`
line after it, so a second entry citing the section, or a figure printed as `Minimal`, fails the
rebuild.

**Every figure is an estimate, and the two editions disagree.** The FY2024-25 edition gives
$50.5 million in each of FY2022 through FY2025, and describes a $750 credit. The FY2026-27 edition
describes the $1,500 joint cap and gives $21.0 million for FY2024, rising to $25.5 million. Both
carry codes B and C and neither carries A, so neither rests on the returns that Table Y-1
tabulates, although the later edition was written after two years of them had been filed.

**A neighboring entry is a different credit.** Entry 2.26 of the FY2024-25 edition is the
nonchartered nonpublic school tuition credit, R.C. 5747.75. It is not the scholarship credit and
is not read.
