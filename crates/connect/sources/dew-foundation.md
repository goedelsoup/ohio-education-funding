# dew-foundation — connector

**Source.** Ohio Department of Education and Workforce: School Finance Payment Reports (SFPR)
and Foundation Funding data files. Predecessor-era files published by the
[Ohio Department of Education](../../../.yidam/corpus/actor/ohio-department-of-education.yml);
current files by [DEW](../../../.yidam/corpus/actor/department-of-education-and-workforce.yml).
The publisher change matters and must be carried on every record.

**Feeds.** [`education-agency`](../../../.yidam/corpus/education-agency/),
[`revenue-stream`](../../../.yidam/corpus/revenue-stream/state-foundation-aid.yml),
[`metric`](../../../.yidam/corpus/metric/), and the scholarship and community school deduction
series behind [`program`](../../../.yidam/corpus/program/).

This is the spine of the numeric corpus. Nearly every per-agency state aid figure originates
here.

## Retrieval interface

```
fetch_payments(fiscal_year, agency_irn?) -> Vec<PaymentRecord>
fetch_adm(fiscal_year, agency_irn?)      -> Vec<MembershipRecord>
fetch_deductions(fiscal_year, program?)  -> Vec<DeductionRecord>
list_agencies(fiscal_year)               -> Vec<AgencyRecord>
```

Every record carries `fiscal_year`, `irn`, `publisher`, `published_at`, and the payment period
the figure was drawn from — SFPR figures are revised across the year, and an early-period
figure is not the annual total.

## Constraints

- Offline mode required: fall back to committed fixtures so analysis stays hermetic.
- Cache on a per-fiscal-year TTL; closed years are immutable and should be cached permanently.
- Publication formats have changed across the period. Expect per-era parsers, not one parser.

## Status

**Wired.** Implemented in [`crates/connect`](../). Three publications are retrieved and **all
three fixtures rebuild from a checkout**, through [`spreadsheet`](../../spreadsheet/) — which
now reads the pre-2007 OLE2 format natively as well as XLSX, so the LibreOffice shim this
connector used to need is gone.

The retrieval interface above was never built as written, and is left as the record of what was
intended. What exists instead is narrower and concrete: `edfund-connect fetch`, a digest
manifest pinning exactly which published file each fixture came from, and column maps in
[`fixtures.rs`](../src/fixtures.rs) rather than record types. The per-payment-period revision
history the interface describes is real and still unretrieved.
