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

## Three decisions the caller must make

**Which index.** CPI-U measures general prices; school costs are majority compensation, for
which the Employment Cost Index is the better deflator but has shorter coverage. Neither is
correct in all cases. The output names the index used, and a figure quoted without it is
incomplete.

**Which window.** An endpoint pair cannot see an interior reversal, and reporting one as though
it characterised the series is the most common way a correctly deflated figure still misleads.
Ohio's per-pupil operating expenditure rises 26% in real terms from FY2000 to FY2022 and falls
about 7% from FY2010 to FY2014 — both computed by this calculator, both true. Two speakers
choosing different windows on one record can contradict each other without either being wrong.
Where a series is available annually, return the shape and not only the endpoints; where only
endpoints exist, say so. See
[`metric/per-pupil-operating-expenditure`](../corpus/metric/per-pupil-operating-expenditure.yml)
and [`crates/deflate/tests/ohio_epp_real_series.rs`](../../crates/deflate/tests/ohio_epp_real_series.rs).

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

## What the financial panel changed

The series ran FY2000–FY2022 until the `dew-five-year-forecast` panel arrived needing FY2023
onward. Four points were added from the same committed BLS extract the check test reads, so they
were verified on the run that added them rather than transcribed and trusted.

Adding them exposed a defect in the check itself. `check_committed_series` iterated a hard-coded
`2000..=2022` rather than the series' own points, so the four new points **fell outside the range
and were checked by nothing** — while the count assertion went on passing, because it counted the
checks rather than the points. It now iterates `CpiSeries::points()`, which makes the check
exhaustive by construction.

The stakes are the reason this matters. Across FY2020–FY2025, CPI-U June rose **25.1%**. Ohio's
statewide district cash balance ends that span **8% above** where it started in nominal dollars
and **14% below** in constant ones. Both are correct and they support opposite arguments — the
same trap this crate's opening example describes, arising again in a panel added twenty phases
later. [verified]
