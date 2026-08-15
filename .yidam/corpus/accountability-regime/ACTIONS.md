# accountability-regime — actions

## Queries

- **Two regimes, one year.** Follow `runs-alongside` to the funding regime in force over the same
  period. A district judged by one system while funded by another is the normal case in Ohio, not
  an edge case — the Fair School Funding Plan phased in across a change of report card scale.
- **Measure provenance.** Follow `measures` to the metric, and check whether the corpus holds it.
  Several report card components have no metric node, so a claim about the overall rating rests
  partly on components this corpus cannot inspect.
- **Consequence ladder.** Follow to `intervention` for what identification leads to, and read
  `fiscal_effect` there rather than inferring it from the regime.
- **What has this node been wrong about?** Read `revisions:`. Each entry carries the claim as
  it stood, what replaced it, and the test or source that settled it — so a withdrawn figure can be
  recognised rather than re-derived, and `found_by` gives the check to re-run.

## Writing a node

- **State the scale and when it changed.** Letter grades and stars are both live in R.C. 3302.10's
  trigger. A rating series that crosses 2022-23 without saying so is two scales in one column.
- **Do not assume the federal regime measures anything federally.** In Ohio it does not; it
  rank-orders the state's rating. Recording the dependency is most of what the node is for.
