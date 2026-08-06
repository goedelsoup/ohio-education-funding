---
name: dispersion
description: Compute equity statistics across Ohio education agencies for a fiscal period — coefficient of variation, McLoone and Verstegen indices, federal range ratio, wealth neutrality
---

# Calculator: dispersion (stub)

**Computes.** The distribution of per-pupil resources across agencies for a fiscal period, and
the standard school-finance equity statistics over it.

**Reads.** [`metric`](../corpus/metric/) definitions;
[`education-agency`](../corpus/education-agency/) series from `crates/`.

**Returns.**

- Coefficient of variation — overall dispersion
- McLoone index — how far the bottom half falls below the median
- Verstegen index — how far the top half rises above it
- Federal range ratio — the spread between the 95th and 5th percentiles
- Wealth neutrality — the association between per-pupil resources and
  [assessed valuation per pupil](../corpus/metric/assessed-valuation-per-pupil.yml)

## Why several statistics and not one

They disagree, and the disagreement is the finding. A reform that raises the bottom improves the
McLoone index and may leave the coefficient of variation unchanged; a reform that constrains the
top does the reverse. Reporting a single dispersion number lets an advocate pick the one that
supports the case. The calculator returns all of them together for that reason.

This is what makes [`doctrine/equity`](../corpus/doctrine/equity.yml) testable rather than
rhetorical — the wealth neutrality figure in particular is the empirical content of the equity
claim.

## Status

**Stub — not implemented.** Blocked on per-agency resource series and on defining a per-pupil
operating expenditure metric, which the corpus names as its largest metric gap. Must be
composed with `deflate` for any cross-period comparison. Implementation lands in
`crates/dispersion/`.
