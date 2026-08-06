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
| [fsfp-input-year-refresh](fsfp-input-year-refresh.yml) | What does freezing FSFP cost inputs cost districts? | Specified, not run |

## Prerequisite

No scenario can be run until the `foundation` calculator exists and the parameter series in
[`parameter/`](../parameter/) are populated. Both are named gaps. Until then this class holds
questions, not answers.
