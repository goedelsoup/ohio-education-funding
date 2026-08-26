# parameter

The dialable numbers. Each node here is one value that a funding regime consumes and the
General Assembly sets, carrying a series across fiscal periods and a link to the enactment
behind each change.

This class exists because of the prediction half of this repository's goal. A policy proposal
in this domain is almost always an edit to one of these values — raise the base cost, change
the phase-in percentage, move the floor — and a counterfactual needs something to bind to.
Parameters are separated from [`formula-component/`](../formula-component/) for that reason:
the component says how the calculation runs, the parameter says what number it runs on, and
only the second is what a proposal changes.

Note the asymmetry with [`metric/`](../metric/). A parameter is an input policy chooses; a
metric is an output policy produces. Confusing the two is the most common way arguments in
this domain go wrong — a base cost per pupil is a decision, a spending per pupil is a
consequence, and they are frequently quoted as though they were the same kind of thing.

**The class used to hold six nodes, and the formula's constants were prose.** 33 distinct
statutory rates and money amounts sat as bare literals inside the `function` fields of the
formula components, with exactly one — the statewide average base cost per pupil — carrying a
node behind it. Ten more nodes now cover the rest, one per statutory section, and each component
declares a `governed-by` edge to the parameters it reads. [verified]

**The value series are still short, and they are short for a reason the class did not expect.**
Almost every section that carries a Fair School Funding Plan parameter states a value for
FY2026 and FY2027 and no other year: three sections apply only to those two by their own terms,
and forty divisions elsewhere read "for fiscal year 2028 and each fiscal year thereafter, an
amount calculated in a manner determined by the general assembly". So a parameter with a
one-biennium series is not an incomplete node — it is a complete reading of a section that
states one biennium. See
[the plan expires and the projection does not](../../decisions/the-plan-expires-and-the-projection-does-not.yml).
[verified]

What is genuinely missing is the *history*: the values these parameters held before FY2026.
Each node carries that as an `unfilled:` entry naming where it would be read from — Ohio Laws'
version archive for a codified rate, the prior budget act for an uncodified one. [open]

See the class definition: [parameter.ont.yml](../parameter.ont.yml).

## Instances

| Node | Unit | Set by |
|------|------|--------|
| [base-cost-per-pupil](base-cost-per-pupil.yml) | dollars per pupil | Biennial budget |
| [twenty-mill-floor](twenty-mill-floor.yml) | mills | H.B. 920, R.C. 319.301 |
| [local-share-charge-off-millage](local-share-charge-off-millage.yml) | mills | Biennial budget |
| [fsfp-phase-in-percentage](fsfp-phase-in-percentage.yml) | percent | Biennial budget |
| [guarantee-funding-base](guarantee-funding-base.yml) | dollars per district | H.B. 166 (2019), and never re-based |
| [appropriation-proration-factor](appropriation-proration-factor.yml) | factor below one | The department, when an appropriation runs short |
| [minimum-state-share](minimum-state-share.yml) | percent | R.C. 3317.017(C), for FY2026-27 only |
| [local-capacity-percentage](local-capacity-percentage.yml) | percent | R.C. 3317.017(A)(4), for FY2026-27 only |
| [special-education-category-multiples](special-education-category-multiples.yml) | ratio | R.C. 3317.013 — the one that does not expire |
| [career-technical-category-multiples](career-technical-category-multiples.yml) | ratio | R.C. 3317.014, through FY2027 |
| [english-learner-category-multiples](english-learner-category-multiples.yml) | ratio | R.C. 3317.016, through FY2027 |
| [preschool-special-education-amounts](preschool-special-education-amounts.yml) | dollars per pupil | R.C. 3317.0213, through FY2027 |
| [gifted-funding-rates](gifted-funding-rates.yml) | dollars per pupil | R.C. 3317.022(A)(6) and 3317.051, through FY2027 |
| [targeted-assistance-rates](targeted-assistance-rates.yml) | ratio | R.C. 3317.0217, for FY2026-27 only |
| [transportation-cost-rates](transportation-cost-rates.yml) | dollars per pupil | Measured by the department under R.C. 3317.0212 |
| [dpia-per-pupil-amount](dpia-per-pupil-amount.yml) | dollars per pupil | R.C. 3317.022(A)(4); the count's blend is the department's |
| [enrolment-supplement-amounts](enrolment-supplement-amounts.yml) | dollars per pupil | H.B. 96 temporary law — in no section |
| [performance-supplement-rate](performance-supplement-rate.yml) | dollars per pupil | H.B. 96 temporary law — in no section |

## The four kinds, and where each one is

The class definition names four: legislated, delegated, measured, uncodified. Until this pass
the instances were almost all legislated, and the taxonomy was mostly an argument. It is not any
more, and each kind now has a node a reader can go and look at.

| Kind | Where to see it |
|------|-----------------|
| legislated | [special-education-category-multiples](special-education-category-multiples.yml) — six digits in a section, and nothing else |
| measured | [transportation-cost-rates](transportation-cost-rates.yml) — trimmed means of what districts reported spending last year |
| delegated | [dpia-per-pupil-amount](dpia-per-pupil-amount.yml) — the 65/35 count blend is in no section; R.C. 3317.03(B)(21) hands it to the department |
| uncodified | [performance-supplement-rate](performance-supplement-rate.yml) — $13 a pupil, on the department's own sheet, alive only in temporary law |
| all four at once | [transportation-cost-rates](transportation-cost-rates.yml) again — the rates are measured, the rider weights legislated, the mass-transit factors delegated, and the 180-day multiplier is in neither |
