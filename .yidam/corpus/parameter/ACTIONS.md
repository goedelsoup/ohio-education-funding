# parameter — actions

## Queries

- **Value in force.** Given a parameter and a fiscal period, return the value and the
  enactment that set it. The `parameter-history` skill.
- **Real-terms series.** Return a dollar-denominated parameter's series in constant dollars
  via `deflate`. A base cost that is nominally flat is falling, and the nominal series does
  not show it.
- **Sensitivity.** Given a parameter, identify which agencies are most exposed to a change in
  it — the input to targeting a scenario.
- **What has this node been wrong about?** Read `revisions:`. Each entry carries the claim as
  it stood, what replaced it, and the test or source that settled it — so a withdrawn figure can be
  recognised rather than re-derived, and `found_by` gives the check to re-run.

## Transitions

- **Revaluation.** A new enactment sets a new value. Appended to `series` with its
  enactment; prior values are never overwritten.
- **Method change.** The parameter's derivation changes rather than its value, as when base
  cost moved from a statewide figure to a district-specific build-up. Recorded as a note in
  `series` and a `replaces` edge between the relevant components.
- **Retirement.** The regime consuming the parameter is superseded. The parameter node
  remains; its series simply ends.

## Skills

- `parameter-history` — assembles the value series and names the enactment behind each change.
- `scenario-run` — perturbs one or more parameters and invokes the simulator.

## Calculators

- `foundation` binds parameters by `simulation_key`.
- `deflate` normalizes dollar-denominated series for cross-era comparison.
