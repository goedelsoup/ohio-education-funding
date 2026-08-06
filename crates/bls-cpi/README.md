# bls-cpi — connector (stub)

**Source.** U.S. Bureau of Labor Statistics: CPI-U for general price deflation and the
Employment Cost Index for the compensation series that dominates school spending.

**Feeds.** [`metric`](../../.yidam/corpus/metric/),
[`fiscal-period`](../../.yidam/corpus/fiscal-period/).

The smallest connector and the one without which nothing else in this corpus is honest. A
repository spanning 1851 to the present cannot compare any two nominal figures, and Ohio's
central mechanism — [H.B. 920](../../.yidam/corpus/legislation/hb-920-1976.yml) holding levy
yields nominally flat — is only visible as a decline once the series is deflated. In nominal
terms it looks like stability.

## Retrieval interface

```
fetch_cpi(series, start_year, end_year) -> Vec<IndexPoint>
fetch_eci(series, start_year, end_year) -> Vec<IndexPoint>
fiscal_year_index(index, fiscal_year)   -> f64   // averaged over Jul-Jun, not calendar
```

`fiscal_year_index` exists because Ohio fiscal years run July to June. Applying a calendar-year
deflator to a fiscal-year figure introduces a systematic half-year error that compounds across
a long series.

## Constraints

- CPI-U is the wrong deflator for school costs, which are majority compensation. ECI is better
  and has shorter coverage. Any deflated figure must name which index produced it.
- Offline mode required; the index series are small enough to commit as fixtures outright.

## Status

Stub. Approved in [decisions/proposals.yml](../../.yidam/decisions/proposals.yml); not
implemented.
