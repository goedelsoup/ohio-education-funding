# Five-year forecast filings — the department's republished submissions

**Source.** Ohio Department of Education and Workforce, School District Five-year Forecasts.
Districts file under R.C. 5705.391; the department republishes every filing as one flat file per
submission window.
**Type.** Primary source, with a qualification that matters — see *What it is not* below.
**Location.** `https://public.education.ohio.gov/School%20District%20Five-year%20Forecasts/`, an
open directory listing. Roughly 5–6 MB per tab-separated file.

**What it contains.** One row per district per forecast line code, about sixty lines each,
covering the general fund only. Each row carries **eight** values: three prior fiscal years as
audited **actuals**, the filing year's own forecast, and four future forecast years. 660 reporting
bodies — Ohio's 609 traditional districts plus joint vocational school districts.

The lines this corpus extracts:

| Code | Line | Why |
|---|---|---|
| `1.010` | General property tax (real estate) | The levy yield actually collected, after H.B. 920 |
| `1.030` | School district income tax | Where one is levied |
| `1.035` | Unrestricted grants-in-aid | State foundation money as the district books it |
| `1.040` | Restricted grants-in-aid | State money it may not spend freely |
| `1.050` | Property tax allocation | The state reimbursing rollback and homestead |
| `1.070` | Total revenue | The district's own receipts |
| `2.080` | Total revenue and other financing sources | The only one that closes the cash identity |
| `5.050` | Total expenditures and other financing uses | |
| `7.010` | Beginning cash balance, 1 July | **Carry-over** |
| `7.020` | Ending cash balance, 30 June | **Cash on hand** |

**Why it matters here.** Every other per-district figure in this repository is modelled — what a
formula computes, what a district spent per pupil on the department's definitions, what its
pupils achieved. **This is the only record of money that changed hands**, and the only measure of
what a district *holds* rather than what it is given.

It also reaches two things that were otherwise blocked. **FY2020** — the year the
[temporary transitional aid guarantee](../corpus/formula-component/temporary-transitional-aid-guarantee.yml)
holds districts at — becomes an observation rather than something inferred from the guarantee
itself, which had made every check of the guarantee circular. And `1.010` is the local side as
*revenue*, which [`tax-abstract`](../../crates/connect/sources/tax-abstract.md) would have
supplied as valuation and still has not.

**Which filings are pinned, and why those.** Two, three years apart, so their actual windows tile
without overlapping: the **FY2023 required spring update** (actuals FY2020–FY2022) and the
**FY2026 required spring update** (actuals FY2023–FY2025). Six consecutive closed years. The
*required* spring window is chosen over the optional fall and final ones because every district
files it.

**Every figure in it is nominal.** The span it covers contains the sharpest price change in forty
years — CPI-U June rose **25.1%** from FY2020 to FY2025 — so a nominal statement about this panel
can have the wrong *sign*, not merely the wrong magnitude. Statewide district cash ends the span
8% above where it started in nominal dollars and 14% below in constant ones. Anything drawn from
this source must say which basis it is in, and
[`crates/deflate`](../../crates/deflate/) is what converts it.

**Caveats:**

- **Actual and forecast are different claims and this corpus extracts only the actuals.** The
  forecast columns are a treasurer's projection made under incentives — a board arguing for a
  levy and a board defending a balance want different numbers on the same line. Treating one as
  a measurement is the characteristic error with this source.
- **Booked aid is not formula output.** `1.035` is what lands in the general fund; the FY2027
  calculator's "total state support" is a different construction. A ratio between them carries
  that gap, and any claim resting on one must be stated against a control measured the same way.
- **General fund only.** Capital, food service, and most federal programmes sit in other funds.
  An expenditure figure here is not the district's total spending.
- **FY2021–FY2024 are the pandemic relief years.** ESSER money was booked in the general fund by
  some districts and separately by others, so a balance rising across that span is not evidence
  about a district's own position. `project::finances::Finances::pandemic_years` names the window
  in code.
- **The two filings can disagree about the instant where they meet.** FY2022's closing balance
  and FY2023's opening balance are the same moment reported by filings three years apart. The
  median gap is **$2** and the 90th percentile **$9,054**, but ten districts restate by more than
  a million and the largest by **$13.3M** (Paint Valley Local). These are reclassifications, not
  errors, and the panel is not a single continuously audited series.
- **One district is short.** Green Local (IRN 049619, Scioto County) is in the FY2027 funding
  model and in the FY2023 filing, and absent from the FY2026 required spring update — so it
  carries FY2020–FY2022 only. Pinned in `project::finances::PARTIAL`.
- **Three published layouts.** Files from FY2022 onward carry a header row. The FY2013–FY2021
  files are headerless with a different column order, and the 2008–2012 files use zero-padded
  fixed-width amounts and a different line-code format. The parser is header-driven and **refuses**
  a headerless file rather than reading it positionally. Extending the panel before FY2020 means
  writing a per-era reader, the same shape of problem as
  [`census-f33`](../../crates/connect/sources/census-f33.md).

The committed extract is
[`crates/project/fixtures/district-finances.csv`](../../crates/project/fixtures/district-finances.csv):
one row per district per closed fiscal year, 3,957 rows. Both source files are digest-pinned in
[`crates/connect/source-digests.txt`](../../crates/connect/source-digests.txt).
