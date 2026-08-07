# scenario

Counterfactuals, as committed knowledge rather than ephemeral tool output. Each node states a
policy question, the parameter perturbations that represent it, the baseline regime it is
computed against, the horizon, and — once run — the per-agency results.

A scenario is a UFO Situation: a configuration of the domain held for inspection, obtaining
under stated assumptions. That framing is the discipline. A projection whose assumptions are
not recorded is not a knowledge claim, it is a number someone produced, and six months later
nobody can say what it assumed. Committing the scenario means the assumptions travel with the
result and both are diffable.

The convention this class enforces: **a scenario is a valid node before it is run.** A
specified-but-unrun scenario states the question and the perturbation and marks its results
`[open]`. Running it is a later epistemic commit against the same node. Seeding an unrun
scenario is therefore not a placeholder — it is the corpus recording which question it intends
to answer, before it has the machinery to answer it.

See the class definition: [scenario.ont.yml](../scenario.ont.yml).

## Instances

| Node | Question | Status |
|------|----------|--------|
| [fsfp-input-year-refresh](fsfp-input-year-refresh.yml) | What does freezing FSFP cost inputs cost districts? | **Run** |

The first scenario has a statewide result. Refreshing the teacher salary input from the FY2022
reference to FY2024 raises **computed** base cost by $465.0 million a year across all 609
districts — but the temporary transitional aid guarantee absorbs $222.9 million of it, so only
**$242.1 million, 52%, is actually delivered.** Two in five districts, holding 41.8% of Ohio's
students, would gain nothing at all.

That correction is worth more than the original number. The node had named the guarantee as its
largest unquantified caveat; quantifying it halved the headline.

Two incidence findings, both structural rather than chosen:

- Districts on formula receive **100%** of any base cost increase; districts at the 5% state
  share floor receive **5%**. A binary split, not a gradient — the transition *is* the floor.
  The node predicted "disproportionate"; the run showed something sharper, and both the
  prediction and the correction are recorded.
- The per-pupil gain ranges from $338.62 to $471.10, a 1.4× spread, driven entirely by the
  six-teacher special minimum binding in **157 of 606** districts.

So a refresh is progressive twice over and a freeze regressive twice over — through the state
share residual, and again through the small-district staffing minimums.

## What running a scenario requires

A scenario needs the calculators its perturbation touches and the parameter values its baseline
rests on. For this one that meant `foundation` complete through all five base cost
sub-components and `local-capacity` for the state share step. Scenarios touching the deduct
mechanism or cross-regime comparison still cannot run — `deduction` and `regime-diff` are
stubs.
