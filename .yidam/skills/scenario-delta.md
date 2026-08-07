---
name: scenario-delta
description: Produce the per-agency winners-and-losers table between a baseline run and a perturbed run, with incidence across wealth and state share
---

# Calculator: scenario-delta

**Computes.** The difference between two funding runs — baseline and perturbed — as a per-agency
table, plus the incidence of that difference across the agency distribution.

**Reads.** Two [`Policy`](../../crates/project/src/policy.rs) values over the district panel;
[`scenario`](../corpus/scenario/) node for the perturbation specification; agency wealth and
state share for the incidence cross.

**Returns.**

- Per agency: dollar delta, percent delta, per-pupil delta
- Incidence: the delta distribution crossed against
  [valuation per pupil](../corpus/metric/assessed-valuation-per-pupil.yml) and
  [state share](../corpus/metric/state-share-percentage.yml)
- Off-formula count: how many agencies were held on a guarantee and therefore unaffected

## The off-formula count is not a footnote

A perturbation is inert for any agency whose payment is set by a guarantee rather than by the
formula. A statewide total that includes them overstates how many districts a change actually
reaches, and during the [Bridge formula](../corpus/funding-regime/bridge-formula.yml) decade
that share was large enough to make the formula substantially advisory.

Report it alongside every total. A result that says "affects all 610 districts" when 200 are
on a guarantee is not a rounding error, it is a different claim.

**Now enforced rather than requested.** There is no method returning a bare dollar figure:
every total is an `Aggregate`, which cannot be constructed without its `Reach`. A caller who
wants the headline number receives the count of untouched districts in the same value.

The inertness is exact, not approximate. A district paid by the guarantee under both policies
moves by **0.000000** — the guarantee is a `max` against a fixed baseline, so the same number
comes out of the same branch twice. For a base cost increase, **256 of 609 districts** are in
that condition. [verified]

## Two orderings, always

The largest dollar movers and the most-affected districts are rarely the same set — a large
urban district and a small rural one can be at opposite ends of the two orderings for the same
policy. Return both, and do not let a caller take only one.

**Also enforced rather than requested**, and the measurement is stronger than the claim. For a
base cost increase the two tables share **no districts at all in their top hundred**: the
largest dollar mover is South-Western City at 21,365 pupils, and the largest per-pupil mover is
Kelleys Island Local at 3.5. [verified]

The disagreement is a property of the lever rather than a law. Removing the guarantee produces
orderings that overlap on 66 of 100, because a guarantee is already close to a per-pupil
quantity. Both are returned in either case. [verified]

## Equal-count bands are wrong on any axis with a floor

138 of 609 districts sit at the minimum state share — 23% of the panel, more than a quintile,
because a floor is a mass point rather than a tail. Equal-count binning cuts that group across a
boundary and reports two adjacent bands with the same axis value and different incidence, which
describes the cut and not Ohio.

Band boundaries are therefore pushed past runs of equal values, and the bands come out uneven.
**"Equal" needs a tolerance:** no district's state share is exactly `0.10` — the 138 spread
upward from `0.099999379565715`, because the fraction is recovered by dividing published dollars
by published ADM. Tie-coherence written as `==` does nothing on the one axis it exists for, and
does it silently. `Axis::resolution` is a required field for that reason. [verified]

## Status

**Implemented** in [`crates/scenario-delta/`](../../crates/scenario-delta/). Findings are pinned
in [`tests/who_a_change_reaches.rs`](../../crates/scenario-delta/tests/who_a_change_reaches.rs);
the report runner is
[`examples/refresh_incidence.rs`](../../crates/scenario-delta/examples/refresh_incidence.rs).

### What it established

**A computed base cost increase is not a funding increase.** Of a $465.0M statewide computed
increase — the department's own figure for refreshing the classroom teacher salary to FY2024 —
**$197.1M, or 42.4%, is delivered as state aid.** [verified]

    computed base cost increase        465.0
      less the ADM denominators         -7.6   base cost averages three years;
                                               the state share is paid on this one
      less the minimum state share    -118.7   138 districts keep a tenth of it
      less the guarantee              -141.6   256 districts keep none of it
    delivered as state aid             197.1

The corpus's prior estimate of what a refresh delivers applies the guarantee and **not** the
minimum state share — recorded as an open item on
[fsfp-input-year-refresh](../corpus/scenario/fsfp-input-year-refresh.yml). That item is now
measured, and the floor is not a small correction.

**The guarantee, not the floor, is what makes a base cost increase progressive.** Across
valuation quintiles the increase pays $305.63 per pupil at the bottom and $20.81 at the top — a
gradient of 0.068 — and the count held on the guarantee rises 10, 31, 39, 88, 88. In the
wealthiest quintile 90 districts are guaranteed and 95 are at the floor, **73 are both**, so the
two mechanisms cannot be added as though independent. In the poorest quintile no district is at
the floor at all. [verified]

**Removing the guarantee is regressive, sharply.** It costs the wealthiest valuation quintile
about seven times per pupil what it costs the poorest, and across state share it takes $1,003.87
per pupil from the districts at the minimum against $160.27 from those the formula supports
most. The guarantee pays the districts the formula says need it least. [verified]

Not monotone at the very top: Q4 loses more per pupil than Q5, because the wealthiest districts
are small and already floored, so there is less distance between formula and baseline to remove.
A claim that the guarantee's benefit rises without limit in wealth is wrong. [verified]

### What it cannot do, and one of these is load-bearing

**No district can be pushed onto the guarantee, and that is a hole in the model.** A guarantee
baseline is a district's FY2020 receipt, and the panel discloses it only for districts *already*
held — being on the guarantee is the only thing that reveals the figure. The **315 districts the
formula pays therefore have no modelled floor at all**, though each has a real one below its
current formula amount. Even a 95% cut lets them fall the full 95% and catches nobody.

So **a simulated cut overstates its own savings**, by an amount this corpus cannot bound. The
districts with no modelled floor hold **46% of Ohio's students**. Any scenario that reduces
formula aid must carry this caveat. [verified] Closing it needs FY2020 per-district payments for
the districts *not* on the guarantee, which no committed source holds. **[open]**

**Typology is not an available axis.** The skill was written to cross incidence against
department typology as well as wealth; no committed fixture carries a typology code, and
[Cleveland Municipal](../corpus/education-agency/cleveland-municipal.yml) records the code as
not yet established. Wealth and state share are implemented; typology is **[open]** pending a
source, and the description above was narrowed to say so rather than promise it.

**A delta is computed at modelled enrollment only.** Running the comparison at projected
enrollment would fold forecast error into a policy effect, and [`project`](project.md) keeps
those apart deliberately. There is no constructor taking a projection.
