# formula-component — actions

## Queries

- **Component decomposition.** Given an agency-year payment, attribute each dollar to the
  component that produced it. This is the `formula-walk` skill's core operation.
- **Cross-regime lineage.** Follow `replaces` to find the predecessor of a component and
  compare the two mechanisms directly rather than comparing regime totals.
- **Parameter dependency.** List every `parameter` a component consumes, to determine what a
  proposed change would actually touch.

## Transitions

- **Replacement.** A new regime introduces a component that occupies the same role. Recorded
  as a `replaces` edge on the successor; the predecessor node is retained.
- **Implementation.** A component gains a `calculator` binding when the corresponding crate
  is written. Until then the component is described but not runnable.

## Calculators

- `foundation` — orchestrates components into a per-agency payment.
- `local-capacity` — implements `fsfp-local-capacity-measure`.
- `millage` — supplies the effective-millage inputs `charge-off-local-share` depends on.
- `regime-diff` — differences two regimes at this level of granularity.

## Gaps

- **The categoricals have no node.** This class holds base cost, local capacity, the charge-off
  it replaced, and the guarantee. It does not hold the six categorical programs — targeted
  assistance, special education, DPIA, English learners, gifted, career-technical — which
  together are **$2.76bn, 43% of formula aid**, larger than any single node here except base
  cost.

  They were invisible because the corpus carried their sum as a residual: core foundation
  funding less the state share of base cost, which is exact and has no parts to describe. The
  six are now extracted per district from `Detail_SFPR` and reconcile to that residual to two
  cents across all 609, so the material for the nodes exists.

  Whether this is one node or six is a real modelling question and not a formality.
  **Targeted assistance is equalisation** — it is the largest of the six at $1.36bn and it falls
  to zero for 135 districts, which makes it behave like local capacity rather than like a
  categorical. **DPIA** is poverty-driven. Grouping them under one node would repeat, at the
  ontology level, exactly the conflation the residual made at the data level.
