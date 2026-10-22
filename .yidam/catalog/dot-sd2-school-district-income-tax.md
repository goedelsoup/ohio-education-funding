---
used-by:
  - ../corpus/revenue-stream/school-district-income-tax.yml
---
# Table SD-2-Q4 — school district income tax distributions, by district

**Source.** Ohio Department of Taxation, tax data series, school district data. Two workbooks:
`sd2qfy24.xlsx` (dated 2 May 2024) and `sd2qfy25.xlsx` (dated 12 May 2025), each the
fourth-quarter table of its fiscal year.
**Type.** Primary source. It is the department's record of money it collected and paid out
under R.C. Chapter 5748.
**Location.** `dam.assets.ohio.gov/raw/upload/tax.ohio.gov/tax_analysis/tax_data_series/
school_district_data/sd2/`. This is the directory pattern [Table SD-1](dot-sd1-school-district-taxes.md)
uses for property taxes. A Wayback CDX listing of the directory shows quarterly files back to
FY1991.

**What it contains.** One row for every district Taxation distributed income tax to in the
fiscal year. Each row gives the county, the school district number, the IRN, the name with a
parenthetical on the tax's term, the year the tax began, the rate, a footnote flag, and the four
quarterly distributions with their total. A `Statewide Total` row sits inside the data block.
FY2024 has 222 rows and $680,506,527.29. FY2025 has 223 rows and $737,718,964.22.

**The flag column carries three meanings.** `(c)` marks the earned-income base of R.C.
5748.01(E)(2). An unflagged row is the traditional base. `(e)` marks a tax that has expired and
is paying out its last withholding. `(d)` and `(f)` mark Ledgemont LSD and Bettsville LSD. Those
districts merged into Berkshire LSD (2015) and Old Fort LSD (2014), and they still receive late
payments under their own IRNs. A row is therefore a payment, not a levy. FY2025 has 210 districts
with a tax in force.

**The rate is a quarter-point.** R.C. 5748.02(B)(1) rounds the Tax Commissioner's estimate to the
nearest one-fourth of one per cent. Every rate in both years is on that grid, from 0.25% to 2%.

**What it is not.** It is not the return data. SD-5 publishes that, by tax year and income
class, in the same directory family, for TY2016 through TY2023. It is not the employer
withholding list either. That list is a calendar-year PDF naming new, expired and changed taxes.
It also does not say how much of a district's revenue the tax is. For that, the five-year
forecast's line 1.030 is the district's own figure.

**Other files in the directory.** FY2026 is published only through its second quarter. The older
years are `.xls` files, or use layouts the extract does not read. `2020-sd-rates.xlsx`, found near
this directory, holds property tax rates, not income tax rates.
