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
- **What has this node been wrong about?** Read `revisions:`. Each entry carries the claim as
  it stood, what replaced it, and the test or source that settled it — so a withdrawn figure can be
  recognised rather than re-derived, and `found_by` gives the check to re-run.

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
- `deflator` — required whenever the horizon spans more than one fiscal period.

## Skills

- `scenario-run` — perturbs, invokes, and commits with provenance.

## Gaps

- **EVERY SCENARIO HOLDS THE STATEWIDE CONSTANTS FIXED. THE HEADLINE ONE NO LONGER DOES, AND WHAT
  IS LEFT IS THREE RESIDUES WITH THREE DIFFERENT REASONS.** The heading used to read "and the
  headline one understates itself by a quarter", in the present tense, four paragraphs above the
  passage recording that it had been fixed. The department's own calculator carries the warning on
  its simulation sheet:

  > Note: The statewide average values (such as economically disadvantaged percentage, median
  > weighted wealth per-pupil or federal median income) are NOT recalculated based on the data
  > changes specific to your district.

  That is a caveat on a tool where a user changes **one** district. It is a larger caveat here,
  where `crates/project::policy` changes **every** district at once, and the corpus had it recorded
  nowhere. [verified — the note is in the FY2027 workbook's shared strings]

  The exposure is concentrated and measurable. `Policy::base_cost_scale` expresses an input-year
  refresh, and **$858m of categorical funding is denominated in the statewide average base cost per
  pupil**: special education, English learners, career-technical and the weighted half of preschool
  special education are each `weight x $8,241.61 x count x state share`. [verified —
  `crates/project/tests/what_a_scenario_holds_fixed.rs`, which recovers the shared multiplicand
  from published amounts to confirm the dependency is real rather than assumed]

  Of that $858m the lever reaches **$812.5m**; the residue is preschool special education's
  weighted half, $45.7m, which sits outside `[H] Foundation Funding` and outside everything a
  scenario computes. [verified — same test, which asserts both parts and their ratio]

  **Sizing the exposure, as a ratio.** Against the FY2018-to-FY2022 restatement, which raised
  statewide base cost about 3.1%:

      3.1% of base cost state share ($3,645.9m)      $113.0m
      3.1% of the reachable categoricals ($812.5m)   $ 25.2m   -- 22% again
      3.1% of gifted's $54.4m as well                $  1.7m   -- 1.5% more

  **These are arithmetic, not aid.** Each is a statewide total times a rate, which is the right
  instrument for a *ratio* — the finding is that the second line is a fifth of the first, and that
  is what a proportion measures. None of the three is what the state would pay. The formula
  departs from `total x rate` twice and in opposite directions: local capacity does not move when
  base cost does, so the state's residual absorbs the whole per-pupil increase and base cost aid
  rises **more** than proportionally; then the guarantee is a `max`, and a district under its floor
  delivers none of the increase. At 3.1% the model delivers $169.1m, against the $113.0m + $25.2m
  a proportional reading gives. [inference — the two departures are `project::policy::apply`; the
  delivered figures are runs of it]

  This table read differently until #251. Its first row was labelled *"a 3.1% refresh, as this site
  models it"*, which described an arithmetic the site has never done; its second and third were
  computed against the whole $858m rather than the $812.5m the lever can reach, so they priced the
  preschool residue the paragraph above excludes. And the block carried the strongest claim tag
  against a test that asserts the totals and the ratio and none of the three dollar figures. The page took the
  first row and printed it as delivered aid, which is #233.

  **The index-driven programs are the ones that cancel**, which is why the exposure is a fifth of
  base cost aid and not three-quarters of it. DPIA divides a district's poverty share by the state's and targeted assistance
  divides by a median: under a change that moves everything, numerator and denominator move
  together. Those two are $1.89bn — larger than the exposed programs — and they are almost
  untouched. That the split falls this way is a consequence of the four mechanism shapes the
  `formula-component` nodes were separated to preserve.

  ~~**This is recorded rather than fixed, deliberately.**~~ **Decided: the site models Ohio.**
  `Policy::base_cost_scale` now scales the denominated categoricals as well as base cost, which is
  a deliberate divergence from the department's own simulator — correctly so, because that tool is
  built for changing one district at a time and this one moves all 609 at once. Recorded at
  [`decisions/scenario-models-ohio`](../../decisions/scenario-models-ohio.yml); every checkpoint in
  the feed was recomputed and the TypeScript `apply()` reproduces them, which is what the
  checkpoints are for.

  **What moved, beyond the arithmetic.** This paragraph is the *other* refresh — the
  FY2022-to-FY2024 restatement the drafts price at 3.95%, not the 3.1% the table above sizes. Both
  reproduce and neither used to say which it was, which is half of how #233 happened.

  At 3.95% a refresh now delivers **$220.6m** of the $497.1m it computes, against $197.1m before —
  44% rather than 42%. It lifts **41** districts off the
  guarantee rather than 38, and reaches 356 rather than 353. And the incidence across state share
  stopped being monotone: the fourth band now collects slightly more per pupil than the fifth,
  because base cost aid rises with the state share while the categorical term is paid on a
  district's own special education, English learner and career-technical counts, which do not sort
  that way. Two mechanisms in one lever. [verified —
  `crates/scenario-delta/tests/who_a_change_reaches.rs`]

  **Three things are still held fixed, and the reasons are not interchangeable.** DPIA and
  targeted assistance ($1.89bn) are indices that genuinely cancel. Gifted ($54.4m) is priced in
  stated salaries, so scaling it by this factor would be an assumption dressed as an identity.
  Preschool special education's weighted half ($45.7m) is denominated identically but sits outside
  `[H] Foundation Funding`, so it is outside everything the scenario computes — not held fixed
  within the model but absent from it. The `/scenario` page gives all three separately rather than
  as one hedge. [verified]

