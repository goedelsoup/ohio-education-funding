# scenario — actions

## Queries

- **Delta table.** Per-agency difference between baseline and perturbed run, sorted by
  magnitude and by percentage of the agency's total. Both orderings are needed: the largest
  dollar movers and the most affected districts are rarely the same set.
- **Incidence.** Cross the delta against agency wealth, typology, and 20-mill-floor status. Who
  gains and loses is the answer; the statewide total is not.
- **Comparison.** Follow `compares-to` between scenarios to isolate the effect of one
  perturbation against another.
- **Sensitivity.** Re-run with the perturbation varied across a range to determine whether the
  result is robust or hinges on the chosen value.

## Transitions

- **Specification → run.** A specified scenario gains results and a `results_path`. This is an
  epistemic commit and must record which calculator version and which parameter series produced
  it, because a scenario re-run against updated inputs is a different claim.
- **Invalidation.** The baseline regime changes, or an input series is corrected. The scenario
  is not deleted; it is marked superseded and the successor links back. Prior projections are
  provenance — including the wrong ones.

## Calculators

- `foundation` — baseline and perturbed runs.
- `scenario-delta` — the difference table.
- `project` — forward input series where the horizon extends past observed data.
- `deflate` — required whenever the horizon spans more than one fiscal period.

## Skills

- `scenario-run` — perturbs, invokes, and commits with provenance.
