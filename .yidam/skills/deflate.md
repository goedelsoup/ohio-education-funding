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

**Stub — not implemented.** Blocked only on `bls-cpi`, whose series are small enough to commit
as fixtures. This is the cheapest calculator to build and a prerequisite for most of the
others — build it first. Implementation lands in `crates/deflate/`.
