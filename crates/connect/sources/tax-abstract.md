# tax-abstract — connector

**Source.** Ohio Department of Taxation property tax abstracts and related tables — assessed
valuation by class, effective operating millage, tax reduction factors, and the data needed to
determine 20-mill floor status.

**Feeds.** [`revenue-stream`](../../../.yidam/corpus/revenue-stream/local-property-tax.yml),
[`parameter`](../../../.yidam/corpus/parameter/twenty-mill-floor.yml),
[`metric`](../../../.yidam/corpus/metric/assessed-valuation-per-pupil.yml).

Without this connector the local half of Ohio school funding is invisible, and the local half
is where the disparities live.

## Retrieval interface

```
fetch_valuation(tax_year, district?)      -> Vec<ValuationRecord>   // by property class
fetch_effective_millage(tax_year, district?) -> Vec<MillageRecord>
fetch_reduction_factors(tax_year, district?) -> Vec<ReductionFactorRecord>
floor_status(tax_year, district)          -> FloorStatus            // at floor | above floor
```

`ValuationRecord` must break out real property from tangible personal property for years
before the [H.B. 66](../../../.yidam/corpus/legislation/hb-66-2005.yml) phase-out completed.
Collapsing the classes produces an apparent valuation collapse in industrial districts that is
a definitional artifact, not an economic event.

## Constraints

- Tax year and fiscal year are offset. Every record carries its tax year; alignment to a
  fiscal period is the caller's decision and must be explicit.
- Offline mode required.
- Pace requests; do not batch-hammer the department's endpoints.

## Status

**Wired**, for one table of the abstracts and two tax years of it.

[Table SD-1](../../../.yidam/catalog/dot-sd1-school-district-taxes.md) — taxable value by class
and real property taxes charged for current expenses, per district, compiled from the DTE-13 and
DTE-14 abstracts — is retrieved for TY2023 and TY2024 and committed as
[`sd1-district-taxes.csv`](../../dispersion/fixtures/sd1-district-taxes.csv). That is the local
half as a *levy*. It is not yet the local half as an effective rate history, and the reduction
factor series the `floor_status` interface above describes is still not retrieved: SD-1 publishes
a calculated Class I rate per district per year, which is a check on
[`millage`](../../millage/) rather than a replacement for the reduction factors it applies.

**What was blocking it, and what was actually true.** The recorded reason was "unstable URLs, one
workbook per table, with the district table's layout changing across years". Two of those three
held and the third did not.

- The layout does **not** change across years. TY2023 and TY2024 carry an identical 28-column
  table. What changes is the *worksheet names* — `ExJVS` and `SD1DATWK23` against `ExJVS24` and
  `SD1DAT24` — and the height of the banner above the header. Matching sheets on their stem and
  finding the header by content covers it, in about ten lines.
- One workbook per table is true, and so is one worksheet per JVSD basis: SD-1 ships the same
  districts twice, with and without the joint vocational operating levy. Both are carried.
- The URLs are unstable, but the harder problem was not named: **`tax.ohio.gov` answers a
  non-browser user agent with a 403 page under a 404 status**, for URLs that resolve in a
  browser. Fetching from `dam.assets.ohio.gov` — the asset host the site's own pages link to —
  gets the same bytes without inspection.

The [District Profile Report](../../../.yidam/catalog/cupp-district-profile-report.md) no longer
has to stand in for valuation, and the 20-mill floor count can now be asked of two years rather
than one.
