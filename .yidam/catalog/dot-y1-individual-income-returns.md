---
used-by:
  - ../corpus/program/scholarship-donation-credit.yml
---
# Table Y-1 — Ohio individual income tax credits by income class

**Source.** Ohio Department of Taxation, Tax Analysis Division. One workbook per tax year:
`Y1TY21.xlsx` for tax year 2021, then `Y1_TY2022.xlsx`, `Y1_TY2023.xlsx` and `Y1_TY2024.xlsx`.
**Type.** Primary source — the department's tabulation of the returns filed with it.
**Location.** `tax.ohio.gov`, Communications → Tax Data Series → Individual Income. The page links
each file under `/static/`, which answers 404 to an agent that is not a browser; the same bytes are
served from `dam.assets.ohio.gov/raw/upload/tax.ohio.gov/communications/tax_data_series/individual_income/y1/`,
and those are the URLs the `tax-income-credits` connector pins.

**What it contains.** Every nonrefundable and refundable credit on the Ohio IT 1040, each as a
dollar amount and a count of returns, by Ohio adjusted gross income class, with a `Total` row.
This corpus reads one credit from it: line 15, the scholarship donation credit of R.C. 5747.73.

**What a figure here is.** The department's note says the amounts are "as claimed on each
individual return before application to income or tax liability", and that they "may overstate
the amount" actually used. A nonrefundable credit cannot reduce tax below zero, so a return with
less liability than its claim forgoes less than it claims. A Y-1 total is therefore an upper
bound on revenue forgone, and never the cost.

**Two layouts.** Through tax year 2023 a credit's name is a merged header over a `Value` /
`Number of Returns` subheader. Tax year 2024 spells each column in full, `Line 15: Scholarship
Donation Credit: Value`. The sheet names change too: `Y1 TY2021`, `Y1 2022`, `Y1 2023`, and
`2024 Nonrefundable Credits`. The reader finds the column by its label in either layout and fails
the rebuild if the year has no `Total` row or more than one.

**Suppression.** Classes with too few returns print `*` in both cells. The fixture keeps the mark,
and the crate reads it as an absence. Tax year 2021 suppresses twenty-four of its forty classes, so a
sum of the printed classes is short of the printed total. No later year suppresses any.
