---
name: formula-walk
description: Decompose one agency-year funding figure into the components and parameter values that produced it, reconciling to the published total before attributing anything
---

# Procedure: formula-walk

**Purpose.** Answer "why this number?" for one agency in one fiscal period, as a chain of
attributable steps rather than a total.

**Composes.** [`foundation`](foundation.md), [`local-capacity`](local-capacity.md),
[`millage`](millage.md), and [`real-dollar`](real-dollar.md) if any comparison is drawn.

## Steps

1. **Check guarantee status first.** A district on temporary transitional aid is paid the
   guarantee, not the formula. Every component figure for it is counterfactual, and a walk that
   does not say so describes a calculation that did not determine the payment.
2. **Reproduce the department's own decomposition and tie out.** `Detail_SFPR` runs `[a] Enrolled
   ADM` through `[N] Total Formula Funding`. Reconcile to the cent before attributing anything.
3. **Base cost build-up**, then the categorical weights, then the supplements.
4. **Apply local capacity** to get the state share — and note that it applies to the weighted half
   only for preschool special education, where the flat $4,000 is paid in full to every district.
5. **Apply the phase-in per line, not once.** A district weighted toward a component with a lower
   percentage receives less than the headline figure.
6. **Name each parameter's `kind`.** A reader cannot tell what it would take to change a number
   until they know whether it is legislated, delegated, measured, or uncodified.
7. **Report the residual.** If the parts do not sum, the gap is the finding.

## What the walk cannot reach

The scholarship and community-school channel is not modelled. The walk explains a foundation
payment, never "the district's funding" — those are different quantities and the second one is
not computable here.

## The rounding rule

Round where the department rounds, and nowhere else. The workspace uses `f64` with explicit
rounding at those points and proves correctness by reproducing published figures to the cent;
there is a test asserting that doing it the other way no longer matches. A walk that re-rounds
will not tie out and the discrepancy will look like a finding.

One known limit: `1.005` is stored just below the midpoint and rounds down. An input landing on a
genuine decimal tie needs decimal arithmetic, not a different rounding mode. Documented and tested
in `edfund-core` rather than hidden.
