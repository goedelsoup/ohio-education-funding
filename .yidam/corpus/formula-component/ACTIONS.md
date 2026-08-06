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
