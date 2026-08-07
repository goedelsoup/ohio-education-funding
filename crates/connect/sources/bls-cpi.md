# bls-cpi — connector

**Source.** U.S. Bureau of Labor Statistics: CPI-U for general price deflation and the
Employment Cost Index for the compensation series that dominates school spending.

**Feeds.** [`metric`](../../../.yidam/corpus/metric/),
[`fiscal-period`](../../../.yidam/corpus/fiscal-period/).

The smallest connector and the one without which nothing else in this corpus is honest. A
repository spanning 1851 to the present cannot compare any two nominal figures, and Ohio's
central mechanism — [H.B. 920](../../../.yidam/corpus/legislation/hb-920-1976.yml) holding levy
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

**Wired.** Implemented in [`crates/connect/src/cpi.rs`](../src/cpi.rs). `fetch_cpi` exists in
the narrow form the deflator needs: one series, one period, parsed from the Bureau's flat file.
`fetch_eci` does not — the Employment Cost Index is the better deflator for school spending and
remains the named gap.

Running it found something. Twenty-one of the twenty-three points in
[`crates/deflate`](../../deflate/) had been transcribed by hand and never checked. Twenty-two
were right; **FY2016 was 241.038 where the Bureau publishes 241.018**. The series is corrected,
every point is now marked verified, and
[`tests/deflator_matches_bls.rs`](../tests/deflator_matches_bls.rs) re-checks all of them on
every test run from a committed extract.

`fiscal_year_index` above proposed a July-to-June average. What is implemented takes the June
observation as the fiscal year's point value. Both are defensible; the choice is recorded in
[`bls-cpi-u`](../../../.yidam/catalog/bls-cpi-u.md) rather than left implicit.
