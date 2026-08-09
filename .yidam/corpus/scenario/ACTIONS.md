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

## Gaps

- **EVERY SCENARIO HOLDS THE STATEWIDE CONSTANTS FIXED, AND THE HEADLINE ONE UNDERSTATES ITSELF BY
  A QUARTER.** The department's own calculator carries the warning on its simulation sheet:

  > Note: The statewide average values (such as economically disadvantaged percentage, median
  > weighted wealth per-pupil or federal median income) are NOT recalculated based on the data
  > changes specific to your district.

  That is a caveat on a tool where a user changes **one** district. It is a larger caveat here,
  where `crates/project::policy` changes **every** district at once, and the corpus had it recorded
  nowhere. [verified — the note is in the FY2027 workbook's shared strings]

  The exposure is concentrated and measurable. `Policy::base_cost_scale` expresses an input-year
  refresh — FY2018 to FY2022 prices raised statewide base cost about 3.1% — and it scales aggregate
  base cost and stops. But **$858m of categorical funding is denominated in the statewide average
  base cost per pupil**: special education, English learners, career-technical and the weighted
  half of preschool special education are each `weight x $8,241.61 x count x state share`.

      a 3.1% refresh, as this site models it       $113.0m
      what it would also move in fact              $ 26.6m   -- 24% more
      counting gifted's salary-priced units        $ 28.3m   -- 25% more

  [verified — `crates/project/tests/what_a_scenario_holds_fixed.rs`, which recovers the shared
  multiplicand from published amounts to confirm the dependency is real rather than assumed]

  **The index-driven programs are the ones that cancel**, which is why the exposure is a quarter
  and not more. DPIA divides a district's poverty share by the state's and targeted assistance
  divides by a median: under a change that moves everything, numerator and denominator move
  together. Those two are $1.89bn — larger than the exposed programs — and they are almost
  untouched. That the split falls this way is a consequence of the four mechanism shapes the
  `formula-component` nodes were separated to preserve.

  **This is recorded rather than fixed, deliberately.** Making `base_cost_scale` also scale the
  denominated categoricals would diverge from the department's own tool, and every checkpoint in
  the feed was computed against the current behaviour. That is a decision about what this site is
  *for* — reproducing the department's simulation, or modelling Ohio — and it wants a decision
  record, not a quiet commit. [open]

  Until then the `/scenario` page states the omission and its size.

