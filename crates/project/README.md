# project — what a different decision would have done

The half of the goal that had no code. `foundation` says what Ohio's formula computes;
this says what it would compute under a different set of choices, and what it will compute as
enrollment moves.

```
cargo run -p project --bin edfund-project -- --guarantee removed
cargo run -p project --bin edfund-project -- --base-cost 1.05 --districts 10
cargo run -p project --bin edfund-project -- --guarantee phase-out:0.5 --through 2032
```

## Simulation and projection are not the same act

Re-running the formula with a changed parameter is deterministic: given the inputs, the answer
is the answer. Projecting enrollment six years out is a forecast.

A run therefore reports **two things and never their sum**. The policy effect is exact and
carries no interval. The enrollment effect is a forecast and carries one. There is no field
holding them added together, because such a number inherits the forecast's error while keeping
the simulation's appearance of precision.

## The levers

| Lever | What it is |
|---|---|
| `--guarantee` | `as-enacted`, `removed`, `rebase:<factor>`, `phase-out:<remaining>` |
| `--base-cost` | multiplier on aggregate base cost — how an input-year refresh is expressed |
| `--min-share` | the minimum state share of base cost |
| `--phase-in`, `--phase-in-cat` | appropriated fraction, base cost and categoricals separately |

The two phase-in dials are separate because Ohio's were. In FY2022 the headline phase-in was
16.67% and Disadvantaged Pupil Impact Aid was phased in at **0%**, so a district's realized
percentage depended on its funding mix and the districts DPIA exists to serve got less than the
headline. One dial cannot express that.

`Policy::current_law()` is the identity, and a test asserts it reproduces the department's own
FY2027 model to the cent for all 609 districts. Without that, no delta means anything.

## Three things building this found

**The minimum state share is 10%, not 5%.** `local-capacity` had 5% hard-coded from the FY2022
worked example. The department's FY2027 calculator states `0.1` on its `Notes` sheet for FY2026
and FY2027 — each biennial budget sets it, and it doubled. **138 of 609 districts sit exactly
on it**, which is a large policy fact the constant concealed.

**Base cost enrolled ADM is the greater of the three-year average and the current year**, not
the average. For 105 districts the published figure is the current year, and in every one of
those the current year is larger. The asymmetry runs one way: a growing district is funded on
this year's students immediately, and a shrinking one keeps two years of students it no longer
has.

**A base cost increase does not pass through proportionally.** The state pays
`base cost per pupil − local capacity per pupil`, and local capacity does not move when base
cost does — so a dollar of extra base cost per pupil is a *dollar* of extra state share per
pupil, not the state's percentage of it. Scaling the state share proportionally understates the
cost of a base cost increase by the whole local share.

## What it cannot do

**Assessed valuation cannot be projected.** There is one observation per district, FY2023. One
point supports no trend, and Ohio valuation does not move smoothly anyway — it steps on a county
reappraisal cycle, six years for a full reappraisal with an update at three. A smooth annual
rate is wrong in every individual year and only roughly right across a cycle.
`Method::Assumed` exists so a caller can supply a rate and have it recorded as *theirs*. Getting
past this needs the [`tax-abstract`](../connect/sources/tax-abstract.md) connector, which is
blocked. Since local capacity is 60% valuation, every projection of the local side of Ohio
school funding is currently an assumption wearing a number.

**Enrollment intervals do not come from the district's own history.** Three observations cannot
estimate a variance. The interval rests on the cross-sectional spread of district growth rates —
how much districts differ from one another, used as a floor — and every interval records that.

**A district at the minimum state share cannot have its base cost pass-through computed**,
because the floor censors the quantity it would be computed from. Those 138 districts are held
at the floor, which understates the gain for any whose local capacity is only just above the
threshold.

**FY2026 is not fully an actual.** The calculator is published in December 2025, so the last
observation in every district's history is partly a departmental estimate. "Observed" is doing
some work there it should not have to do alone.
