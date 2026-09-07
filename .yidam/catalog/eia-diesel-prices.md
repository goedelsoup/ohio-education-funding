# Midwest No. 2 diesel retail prices — the EIA monthly series

**Source.** U.S. Energy Information Administration, No. 2 Diesel Retail Prices, Midwest
(PADD 2), monthly, dollars per gallon. Series `EMD_EPD2D_PTE_R20_DPG`.
**Type.** Primary source — the publishing agency's own machine-readable series.
**Location.** `https://www.eia.gov/dnav/pet/hist_xls/EMD_EPD2D_PTE_R20_DPGm.xls`, about 47 KB.

**What it contains.** One observation per month from **April 1994**, on a two-column sheet:
an Excel date serial and a price. The workbook's other sheet is a table of contents. 389
observations as committed.

**Why it matters here.** [R.C. 3317.0212](../corpus/parameter/transportation-cost-rates.yml)(C)
and (D) set both of the state's transportation rates — $1,337.175 per weighted rider and $6.867
per mile — as **trimmed means of what districts themselves reported spending in the prior fiscal
year**. Neither rate is chosen; both are statistics over district behaviour. Diesel is the part
of that behaviour that moves fastest and the only part with a published monthly price, so this
is the series that says what the rate will do a year before it does it.

**What it shows on arrival.** Ohio is in a compounding fuel shock. On a July-to-June fiscal year:
FY2025 averaged $3.558, FY2026 $4.181 (+17.5%), and FY2027 through August 2026 averages **$5.121**
(+22.5%). The FY2027 rates were computed from FY2026 reported costs and the calculator is dated
16 December 2025, so the rate districts are reimbursed at this year reflects a fuel price roughly
a fifth below what they are paying. That is the lag working exactly as the section specifies.

**Access constraints.** Free, no registration, no API key, no `User-Agent` requirement — unlike
[the Bureau's flat file](bls-cpi-u.md). The EIA's v2 JSON API *does* require a key; the
`hist_xls` path does not, which is why it is the one used.

**Caveats:**

- **It is a legacy BIFF8 workbook, not an XLSX.** Saved by Excel in 2004 and still served in that
  format. [`spreadsheet::ole2`](../../crates/spreadsheet/src/ole2.rs) reads it natively; nothing
  here needs LibreOffice.
- **Retail, not what a district pays.** The price includes federal and state excise tax, and
  school districts buy in bulk and are exempt from some of it. The *level* is therefore wrong
  for a district budget. Percentage *changes* are close but not equal, because a per-gallon tax
  wedge that does not move compresses the proportional swing. Any elasticity taken from this
  should say so. [open — no Ohio school-bus bulk diesel price has been located]
- **PADD 2 is not Ohio.** The Administration publishes no Ohio retail diesel series; the Midwest
  region is the smallest published geography containing it.
- **The first row carries no value.** March 1994 is written with an empty price because the
  weekly survey began part-way through that month. The builder skips valueless rows rather than
  failing, so an absent observation stays absent instead of becoming a build error.
- **Fiscal-year aggregation is a choice, and a different one from the CPI's.** Fuel is bought
  across the whole year, so a July-to-June *mean* is the natural alignment — where
  [the deflator](bls-cpi-u.md) takes a June *point* observation because an index level is not
  consumed. The committed fixture stays monthly so either can be computed.

A committed extract is at
[`crates/connect/fixtures/midwest-diesel-monthly.csv`](../../crates/connect/fixtures/midwest-diesel-monthly.csv).

## Used by

- [`parameter/transportation-cost-rates`](../corpus/parameter/transportation-cost-rates.yml)
- [`formula-component/fsfp-transportation`](../corpus/formula-component/fsfp-transportation.yml)

## Feeds connector

[`eia-diesel`](../../crates/connect/src/registry/eia.rs), built by
[`crates/connect/src/fixtures/diesel.rs`](../../crates/connect/src/fixtures/diesel.rs).
