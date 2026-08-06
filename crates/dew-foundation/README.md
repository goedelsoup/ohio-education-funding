# dew-foundation — connector (stub)

**Source.** Ohio Department of Education and Workforce: School Finance Payment Reports (SFPR)
and Foundation Funding data files. Predecessor-era files published by the
[Ohio Department of Education](../../.yidam/corpus/actor/ohio-department-of-education.yml);
current files by [DEW](../../.yidam/corpus/actor/department-of-education-and-workforce.yml).
The publisher change matters and must be carried on every record.

**Feeds.** [`education-agency`](../../.yidam/corpus/education-agency/),
[`revenue-stream`](../../.yidam/corpus/revenue-stream/state-foundation-aid.yml),
[`metric`](../../.yidam/corpus/metric/), and the scholarship and community school deduction
series behind [`program`](../../.yidam/corpus/program/).

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

Stub. Approved in [decisions/proposals.yml](../../.yidam/decisions/proposals.yml); not
implemented. No `Cargo.toml` yet — this directory records intent and interface only.
