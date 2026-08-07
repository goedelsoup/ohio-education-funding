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

**Declared.** Approved in [decisions/proposals.yml](../../../.yidam/decisions/proposals.yml);
no endpoint wired.

Abstracts are published per tax year at unstable URLs, one workbook per table, and the district
table's layout changes across years. In the meantime the millage and valuation columns of the
[District Profile Report](../../../.yidam/catalog/cupp-district-profile-report.md) stand in —
which covers one year for traditional districts only, and is why the corpus can say how many
districts sit at the 20-mill floor *now* and not how that number moved.
