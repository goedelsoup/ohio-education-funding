---
name: regime-diff
description: Difference two funding regimes at component level for the same agency-year, attributing the change to specific mechanisms rather than reporting a net total
---

# Calculator: regime-diff

**Computes.** For one agency and one fiscal period, what each of two
[`funding-regime`](../corpus/funding-regime/) would pay, decomposed to the
[`formula-component`](../corpus/formula-component/) level.

**Reads.** Both regimes' components and parameters; the agency's input series.

**Returns.** Per component: the amount under each regime and the difference. Plus a residual
line for anything the decomposition does not account for — if the residual is large, the
component mapping between the two regimes is wrong and the result should not be trusted.

## Why component-level

A district that gained under the Fair School Funding Plan gained through a specific mechanism.
It might be the district-specific
[base cost](../corpus/formula-component/fsfp-base-cost-calculation.yml), or the shift from
charge-off to [local capacity](../corpus/formula-component/fsfp-local-capacity-measure.yml), or
the end of the community school deduct. These have different policy implications and different
durability, and a single net delta conflates them.

The `replaces` edge between components is what makes the mapping possible. Where no `replaces`
edge exists, the components are not comparable and the calculator should say so rather than
guess an alignment.

## Status

**Implemented** in [`crates/regime-diff/`](../../crates/regime-diff/), for the one component
pair the corpus can align. Findings are pinned in
[`tests/the_charge_off_against_local_capacity.rs`](../../crates/regime-diff/tests/the_charge_off_against_local_capacity.rs).

### The corpus aligns one component pair, and that is the binding constraint

The `replaces` graph across every `formula-component` node contains **exactly one edge**: FSFP's
local capacity measure replaces the charge-off local share. Base cost, the guarantee, and every
categorical have no declared predecessor. `ALIGNED` and `UNALIGNED` in the crate are that graph,
and a test holds them to it.

So a component-level diff covers one row of a five-row calculation. That is reported rather than
smoothed over, because a decomposition that silently covers a fifth of the formula reads exactly
like one that covers all of it.

### The named first run is not blocked on data — the corpus has no Bridge components at all

This skill has said since genesis that the first useful run is the Bridge formula against the
Fair School Funding Plan for FY2022, and that the blocker was `foundation` supporting both
regimes. That was the wrong diagnosis.
[`bridge-formula`](../corpus/funding-regime/bridge-formula.yml) declares **no `composed-of`
edges whatever** — the corpus models the regime's history, effects, and afterlife, and none of
its mechanisms. There is nothing to align a component against, so no amount of district data
would produce that run.

What the corpus does hold of the Bridge formula is a **scalar per district**: the FY2020 receipt
the FSFP guarantee is anchored to, and only for the 294 districts on the guarantee. Every
existing cross-regime claim in this corpus rests on that scalar, which means each is a net delta
with no attribution — the exact thing this calculator exists to prevent. **[open]**

### What it does run

A counterfactual at FY2027 inputs: the Fair School Funding Plan's own computed base cost held
fixed, with only the local share mechanism substituted. That isolates the aligned component and
is a legitimate reading of "what each of two regimes would pay" — it is **not** a reconstruction
of any year the charge-off governed, which would need that era's formula amount,
cost-of-doing-business factor, and DPIA.

**The reform's stated purpose, measured.** Ottawa Hills Local — high income, valuation below the
state's 40th percentile — is charged 69% more by local capacity than a 23-mill charge-off asks.
Jefferson Township Local — high valuation, low income — is charged 26% less. The district the
charge-off treated as richer is the one local capacity treats as poorer.

**And it is not one-sided:** 413 of 606 districts do better under local capacity, 193 would have
done better under the charge-off. Across valuation quintiles the mean gain per pupil runs
$154.85, $102.38, $107.60, $193.23, $873.92 — every quintile better off, the wealthiest most.

### Reading the residual here

It comes out **exactly zero for 463 of the 470 districts where both sides can be valued**, and
that verifies the substitution rather than discovering anything: holding base cost fixed means
the local share is the only thing that can differ. The seven exceptions are districts where the
charge-off ran past the base cost it was subtracted from, and the residual is exactly that
truncation — the old mechanism had no minimum state share.

A true Bridge-against-FSFP diff would have an enormous residual. Nothing here should be read as
showing the two regimes differ only in this component.

### Where a diff refuses to answer

138 districts sit at the minimum state share, where local capacity is **censored**: all that is
recoverable from published aid is that capacity exceeds a threshold. Their component row is
`None` and their residual is `None`, while their totals stay computable. The difference is
visible and its cause is not, which is the honest state of the comparison rather than a gap to
be filled with a plausible number.
