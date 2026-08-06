# fiscal-period — actions

## Queries

- **Period snapshot.** For one period, return the regime in force, the parameter values, the
  appropriation, and the metrics computed. This is the join the class exists to make possible.
- **Containment.** Follow `part-of` to move between a biennium and its fiscal years — required
  because appropriations are biennial and parameters are annual.
- **Sequence.** Follow `follows` to build a continuous series and to detect gaps. A missing
  period in a series is usually a data gap, not a year in which nothing happened.
- **Cross-period comparison.** Any comparison between two periods must pass through `deflate`.
  Nominal comparison across periods is not a valid operation in this corpus.

## Transitions

- **Closure.** A period ends and actuals replace estimates. Recorded by revising `context`;
  the estimate remains in git history, which is how the corpus distinguishes what was
  projected from what occurred.
- **Supplemental appropriation.** A mid-biennium act changes the appropriation. Recorded as an
  additional `funded-by` edge, not as an edit to the original.

## Calculators

- `deflate` — indexed on fiscal period; the precondition for any cross-period claim.

## Connectors

- `lsc-budget` — appropriations by period.
- `bls-cpi` — deflator series aligned to fiscal rather than calendar years.
