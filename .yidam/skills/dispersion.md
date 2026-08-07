---
name: dispersion
description: Compute equity statistics across Ohio education agencies for a fiscal period — coefficient of variation, McLoone and Verstegen indices, federal range ratio, wealth neutrality
---

# Calculator: dispersion

## Before computing: is the subject a resource or a compensation?

The direction of "better" is not the same, and getting it backwards is easy because the
familiar case is the resource one.

- **Resources** — spending per pupil, total revenue per pupil. Less dispersion is more
  equitable. This is what the indices were designed for.
- **Compensation** — state aid per pupil. Aid exists to offset unequal local capacity, so a
  *wide* spread means strong targeting and a *narrow* one means weak targeting.

Ohio supplies the cautionary case. Realized state aid is measurably more equal than
formula-computed aid — coefficient of variation 0.544 against 0.677, federal range ratio 9.6
against 12.3 — because the
[guarantee](../corpus/formula-component/temporary-transitional-aid-guarantee.yml) tops up
precisely the wealthy districts the formula funds least. Read with the spending habit, that
looks like the system becoming fairer. It is the compensation being flattened.

Every figure this skill produces must name its subject.

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

**Implemented** in [`crates/dispersion/`](../../crates/dispersion/src/lib.rs). 20 tests —
9 unit, 11 integration against real data.

The crate is pure: it takes slices and returns statistics, with no filesystem or network. The
FY2024 District Profile Report extract for all 606 traditional districts is committed as a
fixture and drives the integration tests, so the corpus's empirical equity findings are pinned
and cannot drift silently — a fixture refresh that moves them fails the build rather than
quietly rewriting the conclusion.

Findings now computed rather than asserted: coefficient of variation 0.202, federal range ratio
1.846, median operating expenditure per pupil $15,646; state aid correlates with valuation per
pupil at −0.549 and with economically disadvantaged share at +0.630; local revenue correlates
with valuation above +0.7.

`weighted_mean` is separate from `Dispersion::mean` on purpose: Ohio has enough small districts
that the average district and the average student's district differ by more than $200 per
pupil, and they answer different questions.

**Not yet implemented:** cross-period comparison composed with `deflate`, and dispersion over
anything but the FY2024 cross-section.
