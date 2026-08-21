---
name: scenario-run
description: Perturb a parameter set, run baseline against perturbed, and commit the result as a scenario node with its reach, incidence, and provenance
---

# Procedure: scenario-run

**Purpose.** Turn a proposed change into a per-agency table that a later reader can reproduce, and
commit it as a [`scenario`](../corpus/scenario/) node.

**Composes.** [`foundation`](foundation.md), [`scenario-delta`](scenario-delta.md),
[`project`](project.md) where enrollment moves, [`real-dollar`](real-dollar.md) for any figure
compared across years.

## Steps

1. **Bind to a `simulation_key`.** A scenario that cannot name the parameter it moves cannot be
   re-run.
2. **Check the parameter's `kind` first.** Perturbing a *measured* parameter directly is usually
   a modelling error: nobody sets it, so "what if it were different" has no policy corresponding
   to it. What a policy can change is the measurement — which reference year is read — one level
   up. The [cost input refresh](../corpus/scenario/fsfp-input-year-refresh.yml) is exactly this
   shape.
3. **Run both against the same fixtures.**
4. **Take the delta table with its reach.** A total cannot be constructed without the count of
   districts the change fails to reach; `scenario-delta` returns both orderings or neither, on
   purpose, so that a report cannot quietly show only the winners.
5. **Report incidence across wealth and state share**, not just the aggregate.
6. **Commit the node with the `establish:` verb** — a node authored is new understanding —
   carrying inputs, deltas, reach, incidence, and links to the parameter and regime it binds
   to. The verb is named here rather than a document away because this is the only point at
   which the next commit can still be caught; see the commit vocabulary in
   [`GRAPH.md`](../.vendor/prelude/GRAPH.md).

## The three things that make a scenario overstate itself

**The guarantee.** A district held on temporary transitional aid is insulated from most parameter
changes, and the phase-in percentage in particular is inert for it. The share of districts
insulated is required context; omitting it makes every scenario look more powerful than it is.

**One year of fixtures.** The department publishes one foundation calculator at a time and
replaces rather than archives it. A multi-year scenario built on that is a projection wearing a
simulation's clothes.

**The channels outside the model.** A scenario says what the formula would pay. It does not say
what a district would receive, because the scholarship and capital channels are not here.

## Simulation and forecast do not add

[`project`](project.md) reports them separately and refuses to sum them. A simulation says what a
rule would have produced; a forecast says what is expected to happen. A single number combining
them is not a quantity.

## On divergence from the department's tool

This simulator models Ohio rather than mirroring the department's simulator, and diverges
deliberately on `base_cost_scale`, which moves the $812m of categorical funding priced into the
statewide average base cost per pupil. State the divergence and its size in the scenario node.
A result that silently disagrees with the official tool will be read as an error in this one.
