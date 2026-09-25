# FY2019 Final Traditional District Foundation Payment Report

**Source.** Ohio Department of Education and Workforce, Office of Budget and School Funding.
One workbook, `FY19_SFPR_FIN_2.xlsx` — the second and final settlement of fiscal year 2019.
**Type.** Primary source — what the department paid each district under the formula H.B. 110
replaced, not a report about it.
**Location.** `education.ohio.gov`, Finance and Funding → School Payment Reports → State Funding
for Schools → Traditional School Districts → Districts Payment Reports in Excel Format. Openly
served, no credentials, 631,833 bytes.

**Why a 2019 file answers a 2026 question.** `[K] Formula Transition Supplement` holds a district
at `[L1]`, a FY2021 funding base, and `[L1]` reaches this corpus as a single published column. Ask
what retiring the temporary transitional aid guarantee `[I]` does and there is a third reading
besides "`[K]` stands" and "`[K]` is repealed": recompute `[K]` against a base that never contained
a guarantee either. That needs the guarantee broken out of `[L1]`, and the FY2021 payment report
**carries no guarantee line at all** — the guarantee had been folded into the base by then. This
file is where the figure still exists, per district, as `TRANSITIONAL GUARANTEE`: one of the
thirteen capped components that sum to `TOTAL FOUNDATION FUNDING`. Issue #394 recorded the split as
blocked on credentialed access to the deduct-era reports. It was blocked on reading one year
earlier.

**What it contains.** One sheet, `FY19_SFPR_FIN_2`, 133 columns wide and 621 rows deep, of which
612 are districts — **there is no statewide aggregate row**, so a total is a sum this corpus takes
and not one the department published. The label block occupies rows 2 through 9 and data begins at
row 11. The thirteen components run left of `TOTAL FOUNDATION FUNDING` at column 28, and the
guarantee and cap machinery runs to its right: `FUNDING BEFORE GUARANTEE`, `FUNDING WITHIN
GUARANTEE`, `GUARANTEE BASE`, `CAP RATIO`, and a second copy of the total as `TOTAL FUNDING` 101
columns further on. `docProps` credits Shams, Daria and dates creation and last modification both
to 2019-12-24. No sheet is protected.

**Four identities hold to the cent on all 612 rows,** and
[`connect::fixtures::transition_base`](../../crates/connect/src/fixtures/transition_base.rs)
refuses the file if any stops holding: the thirteen components sum to `TOTAL FOUNDATION FUNDING`;
`TOTAL FUNDING` is that same total; `FUNDING BEFORE GUARANTEE` less `FUNDING WITHIN GUARANTEE` is
exactly `CAREER TECH EDUCATION FUNDING`; and **no district is both guaranteed and capped** — 335
draw the guarantee, 164 sit under the cap, and the two sets do not intersect. The third of those is
substantive rather than bookkeeping: the FY2019 guarantee did not protect career-technical funding.

A 612-row extract is committed at
[`crates/project/fixtures/fy19-payment-report.csv`](../../crates/project/fixtures/fy19-payment-report.csv).

## Caveats

- **`GRADUATION BONUS` is published twice under an identical joined label,** at columns 24 and 115
  — once as a paid component inside the total, and once among the measures the bonus derives from.
  No spelling distinguishes them. The extractor resolves components to the left of
  `TOTAL FOUNDATION FUNDING` and the guarantee columns to its right, because a component is a
  column the total is the sum of and the report prints those first.

- **This report carries one district the FY2021 base sheet does not.** Newbury Local SD (`047217`)
  dissolved into West Geauga Local SD (`047225`), and the department carried its funding forward
  rather than dropping it: West Geauga's `[L1]` first term exceeds its own FY2019 total by exactly
  Newbury's $1,048,132.43.

- **`TRANSITIONAL GUARANTEE` is not recoverable from `GUARANTEE BASE` and the funding columns.**
  Both plausible identities fail — `max(GUARANTEE BASE − FUNDING BEFORE GUARANTEE, 0)` on 340
  districts and `max(GUARANTEE BASE − FUNDING WITHIN GUARANTEE, 0)` on 77. Read the paid column.

- **`FUNDING BEFORE GUARANTEE` plus `TRANSITIONAL GUARANTEE` is the total only where the cap does
  not bind** — it fails on 163 of the 612, which is approximately the 164 capped districts. The
  cap is applied after the guarantee, not alongside it.

- **The per-year traditional-district reports are openly served back to FY2016.** Only the
  department's *Foundation Legacy Payment Reports (1999–2021)* collection gates on OH|ID, and it
  overlaps the open era it is needed for. See
  [`dew-payment-reports`](../../crates/connect/sources/dew-payment-reports.md).
