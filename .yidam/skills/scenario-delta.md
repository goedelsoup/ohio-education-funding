---
name: scenario-delta
description: Produce the per-agency winners-and-losers table between a baseline run and a perturbed run, with incidence across wealth and typology
---

# Calculator: scenario-delta (stub)

**Computes.** The difference between two `foundation` runs — baseline and perturbed — as a
per-agency table, plus the incidence of that difference across the agency distribution.

**Reads.** Two `foundation` outputs; [`scenario`](../corpus/scenario/) node for the
perturbation specification; agency wealth, typology, and floor status for the incidence cross.

**Returns.**

- Per agency: dollar delta, percent delta, per-pupil delta
- Incidence: the delta distribution crossed against
  [valuation per pupil](../corpus/metric/assessed-valuation-per-pupil.yml), typology, and
  [state share](../corpus/metric/state-share-percentage.yml)
- Off-formula count: how many agencies were held on a cap or guarantee and therefore unaffected

## The off-formula count is not a footnote

A perturbation is inert for any agency whose payment is set by a guarantee rather than by the
formula. A statewide total that includes them overstates how many districts a change actually
reaches, and during the [Bridge formula](../corpus/funding-regime/bridge-formula.yml) decade
that share was large enough to make the formula substantially advisory.

Report it alongside every total. A result that says "affects all 610 districts" when 200 are
on a guarantee is not a rounding error, it is a different claim.

## Two orderings, always

The largest dollar movers and the most-affected districts are rarely the same set — a large
urban district and a small rural one can be at opposite ends of the two orderings for the same
policy. Return both, and do not let a caller take only one.

## Status

**Stub — not implemented.** Depends on `foundation`. Implementation lands in
`crates/scenario-delta/`. The first run is
[fsfp-input-year-refresh](../corpus/scenario/fsfp-input-year-refresh.yml).
