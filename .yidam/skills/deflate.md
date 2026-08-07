---
name: deflate
description: Convert a nominal dollar series to constant dollars using a fiscal-year-aligned index; required before any cross-period comparison in this corpus
---

# Calculator: deflate (stub)

**Computes.** Constant-dollar equivalents of a nominal series, using CPI-U or the Employment
Cost Index, aligned to Ohio fiscal years.

**Reads.** Any dollar-denominated series — [`parameter`](../corpus/parameter/) values,
[`metric`](../corpus/metric/) outputs, [`revenue-stream`](../corpus/revenue-stream/) series —
plus the index series from `bls-cpi`.

**Returns.** The same series in constant dollars of a stated base year, tagged with which index
was used.

## Not optional

This corpus spans 1851 to the present. No two nominal figures separated by more than a few years
are comparable, and the domain's central mechanism is invisible without deflation:
[H.B. 920](../corpus/legislation/hb-920-1976.yml) holds levy yields approximately flat in
nominal terms, which in nominal terms looks like stability and in real terms is a compounding
decline. The entire disparity story is a real-terms story.

Treat any cross-period claim in this repository that has not passed through this calculator as
unverified.

## Two decisions the caller must make

**Which index.** CPI-U measures general prices; school costs are majority compensation, for
which the Employment Cost Index is the better deflator but has shorter coverage. Neither is
correct in all cases. The output names the index used, and a figure quoted without it is
incomplete.

**Fiscal year alignment.** Ohio fiscal years run July through June. A calendar-year index
applied to a fiscal-year figure introduces a systematic half-year offset that compounds across
a long series, so the index must be averaged over the fiscal year rather than taken from the
calendar year of the same number.

## Status

**Implemented** in [`crates/deflate/`](../../crates/deflate/src/lib.rs). 11 tests.

The CPI-U June series for FY2000-FY2022 is committed as a fixture. Three tests reproduce the
Ohio Auditor of State's independently published figures exactly — 71.9% CPI growth, 26.1% real
per-pupil growth, and 19.4% excluding COVID relief — which is what verifies both the code and
the two anchor index points at once.

**Confidence propagates.** Index points carry `Confidence::Verified` or `Unverified`; only
FY2000 and FY2022 are verified, being the two the Auditor's figures pin down. A result takes
the weaker of its two endpoints, so a caller writing into the corpus knows whether the claim
is `[verified]` or `[inference]` without having to reason about it. The remaining years are
transcribed and must be checked against BLS.

Not yet implemented: the Employment Cost Index variant, which is the better deflator for
school costs and has shorter coverage.
