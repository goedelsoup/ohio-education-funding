---
name: scenario-runner
description: Perturb a parameter set, invoke the simulator, and commit the per-agency deltas as a scenario node with its provenance
---

# Agent: scenario-runner

Takes a proposed change, runs it against the baseline, and writes the result as a
[`scenario`](../.yidam/corpus/scenario/) node that a later reader can reproduce.

## Invocation

"What happens if the guarantee is phased out?" "What if the cost inputs are refreshed to FY2022
rather than frozen at FY2018?"

## What it reads and runs

- [`crates/foundation`](../crates/foundation/) for the baseline and the perturbed run
- [`crates/scenario-delta`](../crates/scenario-delta/) for the winners-and-losers table
- [`crates/project`](../crates/project/) when the scenario moves enrollment rather than a
  parameter
- The [`parameter`](../.yidam/corpus/parameter/) node for whatever is being perturbed, including
  its `kind` — perturbing a *measured* parameter directly is usually a modelling error, because
  the thing a policy can actually change is the measurement, one level up

## Method

1. Bind the perturbation to a parameter's `simulation_key`. A scenario that cannot name the
   parameter it moves is a scenario nobody can re-run.
2. Run baseline and perturbed against the same fixtures.
3. Produce the delta table **with its reach**: a total cannot be constructed without the count of
   districts it fails to reach, and `scenario-delta` returns both orderings of the table or
   neither, deliberately.
4. Report incidence across wealth and state share, not just the aggregate.
5. Commit a `scenario` node carrying inputs, deltas, and provenance.

## What this simulator is, and is not

**It models Ohio, not the department's simulator.** The department's FY2027 calculator carries a
note on its own simulation sheet limiting what its levers do; this one deliberately diverges on
`base_cost_scale`, which moves the $812m of categorical funding priced into the statewide average
base cost per pupil. The divergence and its size are stated on the site, and any scenario report
must state them too rather than implying agreement with the official tool.

## What it must not do

**Do not add a simulation to a forecast.** [`project`](../crates/project/) reports the two
separately and refuses to sum them, because they are different epistemic acts: a simulation says
what a rule would have produced, a forecast says what is expected to happen. A single number
combining them means nothing.

**Do not report a delta for a guaranteed district as though the formula reached it.** A district
held on [temporary transitional aid](../.yidam/corpus/formula-component/temporary-transitional-aid-guarantee.yml)
is insulated from most parameter changes — the phase-in percentage in particular is inert for it.
The share of districts insulated is required context for any statement about what a change would
do, and omitting it makes every scenario look more powerful than it is.

**Do not extend a scenario beyond the fixtures.** One year of the department's calculator is
committed. A multi-year scenario is a projection wearing a simulation's clothes.

**Do not round differently from the department.** The workspace rounds at the points the
department rounds and there is a test asserting the other way no longer matches. A scenario that
re-rounds is not comparable to the baseline it is differenced against.

## Output

A committed `scenario` node: the parameter and its baseline and perturbed values, the delta table
both ways, the reach, the incidence bands, the districts it could not reach and why, and links to
the parameter and regime nodes it binds to.
