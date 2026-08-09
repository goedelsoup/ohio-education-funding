# Table SD-1 — School district taxable value and taxes charged

**Source.** Ohio Department of Taxation, Tax Analysis Division. `SD1CY21.xlsx` through
`SD1CY24.xlsx` (the last dated 3 April 2026), from the Tax Data Series.
**Type.** Primary source — compiled from the DTE-13 and DTE-14 abstracts county auditors file
with the department.
**Location.** `tax.ohio.gov`, Researcher → Tax Analysis → Tax Data Series → Property Tax.

**What it contains.** Every Ohio school district, once per tax year: taxable value broken out by
class (agricultural, residential, mineral, industrial, commercial, railroad, public utility),
real property taxes charged for **current expenses**, calculated Class I and Class II tax rates,
average value per pupil, and the fiscal-year ADM the per-pupil figure divides by.

This is the local half of the funding picture, and until it was retrieved the corpus had no
per-district measure of it except the one-year millage and valuation columns of the
[District Profile Report](cupp-district-profile-report.md).

**Why four tax years and not two.** Two give a level and a change, which is what the tax page
reads. The fourth year is here for a different reason: Ohio's counties reappraise or update on a
staggered three-year cycle, so **TY2022–TY2024 contains exactly one valuation event for every one
of the 88 counties**, and TY2021 makes the earliest of those a change rather than a level. That
window is what lets each district's reappraisal be measured against its own quiet years, which is
how [recognized valuation](dot-reappraisal-calendar.md) is reconstructed without a source that
publishes it. The separation is stark — a median 28.6% jump in the event year against 1.5%
otherwise.

**A trap the extra years set, recorded because it caught four callers.** With two tax years,
`first()` and `last()` are adjacent, so every consumer that wanted a year-over-year change took
the ends. With four they are three years apart and span a reappraisal. Nothing failed loudly:
a rate is still a rate and a page still renders. Anything reading this fixture as a change must
take the **last two**.

**Two worksheets per workbook, differing in one thing.** `SD1DAT…` counts the joint vocational
school district's operating levy in the taxes charged; `ExJVS…` removes it. 501 of 611 districts
are in a JVSD, and the difference is **$513 million** in TY2024. A figure quoted without saying
which basis it is on is ambiguous by roughly 4% of the state total, so
[the fixture carries both](../../crates/dispersion/fixtures/sd1-district-taxes.csv).

**Three things "taxes charged" is not.** Each has produced a wrong number somewhere, and each is
recorded on [`build_sd1_extract`](../../crates/connect/src/fixtures.rs) as well as here.

- **Not net of state reimbursement.** The department's own note: the figures "include taxes that
  have been reduced under various property tax programs that are reimbursed to local school
  districts by the state" — the non-business credit, owner-occupancy credit, and homestead
  exemption. About a tenth of what this column calls property tax is state money. Using it as
  the local share overstates local effort, and by more in districts with more owner-occupied
  housing.
- **Not a fiscal year.** A TY2024 charge is collected across calendar 2025 — half early in the
  year, half in July — so it straddles FY2025 and FY2026. Any ratio against a single-fiscal-year
  denominator counts money the district had not yet received, and the error is largest exactly
  where a levy has just passed.
- **Not collected.** Delinquency puts receipts below the charge by a district-specific amount
  this table cannot see. The [five-year forecast](dew-five-year-forecast.md)'s line `1.010` is
  the receipt side, and the two are worth differencing rather than substituting.

**Access constraints.** Freely available, ~470 KB XLSX. **`tax.ohio.gov` answers any non-browser
user agent with a 403 page served under a 404 status**, including for URLs that resolve
perfectly well in a browser — so the connector fetches from `dam.assets.ohio.gov`, the asset host
the site itself links to, where the same bytes are served without inspection. The mapping is
`tax.ohio.gov/static/X` → `dam.assets.ohio.gov/raw/upload/tax.ohio.gov/X`; note `raw/upload` for
workbooks against `image/upload` for the older PDFs.

**Caveats.**

- **The worksheet names drift and the layout does not.** `ExJVS` and `SD1DATWK23` in the TY2023
  workbook against `ExJVS24` and `SD1DAT24` in TY2024, over an identical 28-column table. The
  banner above the header is also one row shorter in TY2023. The connector matches sheets on
  their stem and finds the header by content for that reason.
- **The IRN is stored as a number**, so Manchester Local's `000442` arrives as `442`. Joining
  that against the department of education's zero-padded keys matches nothing, and fails as an
  empty join rather than an error.
- **`N/A` in TY2024 and `NA` in TY2023** mean the same thing — two spellings of not-applicable
  from one publisher two years apart. Both appear only in the value-per-pupil *rank* column, on
  the two districts with no ADM, and neither is extracted.
- **The workbook is revised in place.** The TY2024 file carries an internal date of 3 April 2026,
  later than the analyses circulating from it in August 2025. The digest manifest pins which
  bytes the fixture was built from; a mismatch means the department restated, not that the
  fixture drifted.
- **611 districts**, against the 609 in the department of education's FY2027 model and the 607 in
  the report card files. The extra rows are the island districts, which have valuation and no
  ADM.

## What it settled

Policy Matters Ohio's August 2025 brief on property tax repeal compared each district's TY2024
real property taxes charged to its FY2025 operating expenditure. Reproduced from this table and
[the report card spending file](dew-report-card-spending.md), the published figures come back
exactly: statewide median 36%, fiftieth-ranked district 74%, Columbus 47.8%, Cleveland 38.8%,
Cincinnati 50.4%, and $513 million of JVSD operating levy. See
[`crates/dispersion/tests/sd1_district_taxes.rs`](../../crates/dispersion/tests/sd1_district_taxes.rs).

## Used by

- [`crates/dispersion/fixtures/sd1-district-taxes.csv`](../../crates/dispersion/fixtures/sd1-district-taxes.csv)

## Feeds connector

[`tax-abstract`](../../crates/connect/sources/tax-abstract.md)
