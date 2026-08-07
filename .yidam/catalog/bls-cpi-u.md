# CPI-U all items — the Bureau of Labor Statistics flat file

**Source.** U.S. Bureau of Labor Statistics, Consumer Price Index for All Urban Consumers, all
items, U.S. city average, not seasonally adjusted. Series `CUUR0000SA0`.
**Type.** Primary source — the publishing agency's own machine-readable series.
**Location.** `https://download.bls.gov/pub/time.series/cu/cu.data.1.AllItems`, about 2.7 MB.

**What it contains.** Every observation of every all-items CPI series the Bureau publishes, as
one tab-separated file: series identifier, year, period (`M01`–`M12`, plus `S01`/`S02`
semi-annual and `M13` annual), value, and footnote codes. Monthly coverage of `CUUR0000SA0`
begins in **1913**, which is 62 years short of this corpus's 1851 starting point but covers
every year for which Ohio per-pupil figures exist.

**Why it matters here.** A repository spanning 1851 to the present cannot compare any two
nominal figures without it, and Ohio's central mechanism —
[H.B. 920](../corpus/legislation/hb-920-1976.yml) holding levy yields nominally flat — is only
visible as a *decline* once a series is deflated. In nominal terms it looks like stability.

**What it corrected.** [`crates/deflate`](../../crates/deflate/) carried a hand-transcribed
CPI-U June series in which twenty-one of twenty-three points had never been checked against the
agency. Comparing them to this file confirmed twenty-two and found one wrong: **FY2016 was
transcribed as 241.038, and the published value is 241.018.** The whole series is now marked
[verified] and `crates/connect/tests/deflator_matches_bls.rs` re-checks it on every test run.

**Access constraints.** Free, no registration, no API key. But the Bureau **rejects any request
whose `User-Agent` does not carry a contact address**, with a bare `403` and no explanation — a
browser-like agent string is refused as well. Set `EDFUND_CONTACT` to an email address before
fetching; `crates/connect` builds the agent string from it and says so when a fetch 403s.

**Caveats:**

- **Period matters as much as series.** `M06` is June; `M13` is the annual average and `S01`
  is the first-half average. Reading the wrong period gives a plausible number that is not the
  one the deflator's fiscal-year alignment assumes.
- **Neighbouring series names are close enough to confuse.** `CUUS0000SA0` is the semi-annual
  companion and `CUUR0000SAF1` is food. Both would deflate quietly and wrongly.
- **CPI-U is a general consumer index.** School costs are majority compensation, for which the
  Employment Cost Index is the better deflator with much shorter coverage. Any figure produced
  with this must name the index used — see the note in `crates/deflate`.
- **Fiscal-year alignment is a choice.** This corpus takes the June observation as the point
  value for a fiscal year ending 30 June. A July-to-June average is the defensible alternative
  and would move long-series growth figures slightly.

A committed extract of the June observations is at
[`crates/connect/fixtures/cpi-u-june.tsv`](../../crates/connect/fixtures/cpi-u-june.tsv), which
is what makes the deflator's verification runnable without a network.

## Used by

- [`metric/per-pupil-operating-expenditure`](../corpus/metric/per-pupil-operating-expenditure.yml)
- [`fiscal-period/`](../corpus/fiscal-period/)
- [`legislation/hb-920-1976`](../corpus/legislation/hb-920-1976.yml)

## Feeds connector

[`bls-cpi`](../../crates/connect/sources/bls-cpi.md), implemented in
[`crates/connect/src/cpi.rs`](../../crates/connect/src/cpi.rs).
