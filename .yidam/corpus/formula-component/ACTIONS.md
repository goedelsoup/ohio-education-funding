# formula-component — actions

## Queries

- **Component decomposition.** Given an agency-year payment, attribute each dollar to the
  component that produced it. This is the `formula-walk` skill's core operation.
- **Cross-regime lineage.** Follow `replaces` to find the predecessor of a component and
  compare the two mechanisms directly rather than comparing regime totals.
- **Parameter dependency.** List every `parameter` a component consumes, to determine what a
  proposed change would actually touch.
- **What has this node been wrong about?** Read `revisions:`. Each entry carries the claim as
  it stood, what replaced it, and the test or source that settled it — so a withdrawn figure can be
  recognised rather than re-derived, and `found_by` gives the check to re-run.

## Transitions

- **Replacement.** A new regime introduces a component that occupies the same role. Recorded
  as a `replaces` edge on the successor; the predecessor node is retained.
- **Implementation.** A component gains a `calculator` binding when the corresponding crate
  is written. Until then the component is described but not runnable.

## Calculators

- `foundation` — orchestrates components into a per-agency payment.
- `local-capacity` — implements `fsfp-local-capacity-measure`.
- `millage` — supplies the effective-millage inputs `charge-off-local-share` depends on.
- `regime-diff` — differences two regimes at this level of granularity.

## Gaps

- ~~**The categoricals have no node.**~~ **CLOSED** — six nodes written, listed in the README
  with the four shapes they fall into. What follows is the record of why it took eight phases and
  what the modelling question turned out to be, kept because the reasoning outlived the gap.

  The original entry:

- **The categoricals have no node.** This class holds base cost, local capacity, the charge-off
  it replaced, and the guarantee. It does not hold the six categorical programs — targeted
  assistance, special education, DPIA, English learners, gifted, career-technical — which
  together are **$2.76bn, 43% of formula aid**, larger than any single node here except base
  cost.

  They were invisible because the corpus carried their sum as a residual: core foundation
  funding less the state share of base cost, which is exact and has no parts to describe. The
  six are now extracted per district from `Detail_SFPR` and reconcile to that residual to two
  cents across all 609, so the material for the nodes exists.

  Whether this is one node or six is a real modelling question and not a formality.
  **Targeted assistance is equalisation** — it is the largest of the six at $1.36bn and it falls
  to zero for 135 districts, which makes it behave like local capacity rather than like a
  categorical. **DPIA** is poverty-driven. Grouping them under one node would repeat, at the
  ontology level, exactly the conflation the residual made at the data level.

  **THE MODELLING QUESTION IS NOW ANSWERABLE, AND THE ANSWER IS SIX NODES IN FOUR SHAPES.** All
  six are decomposed to their inputs and reproduce the department's per-district amounts from
  the workbook's own formulas. They are not variations on one mechanism:

      shape              components                              parameters the node consumes
      weight x count     special education (6 ascending),        a weight vector, a base cost
                         career-technical (5, plus an
                         associated-services weight),
                         English learners (3 *descending*)
      blend + index      DPIA — two poverty counts at 65/35,     two blend weights, a per-pupil
                         indexed on the statewide share and      amount, a statewide share
                         *squared*
      units + floors     gifted — two per-pupil amounts and      two per-pupil rates, three unit
                         three unit kinds, each clamped          prices, three divisors, a floor
                                                                 pair and a cap
      equalisation       targeted assistance — a capacity tier   a wealth blend, two rates, three
                         and a wealth tier, added                ADM brackets, an index floor

  Three consequences for the ontology, each of which the single node would have hidden:

  1. **CTE does not share a base cost with the other weighted programs.** Its weights multiply
     $9,855.62 where special education's and English learners' multiply $8,241.61. A shared
     `base-cost-per-pupil` parameter edge from all three would be wrong.
  2. **Targeted assistance belongs beside `fsfp-local-capacity-measure`, not beside the
     weights.** Both blend valuation with federal adjusted gross income — 60/40 here against the
     capacity measure's own weights — and both are about the tax base rather than the pupil. Its
     `replaces` edge probably reaches the charge-off era's equalisation, not any categorical.
  3. **Gifted's parameters are prices, not rates.** $85,776 a coordinator unit is a salary
     assumption, so it belongs to the same family as the base cost build-up's staffing costs and
     will move when those are refreshed. Nothing else among the six behaves that way.

  Also to record when the nodes are written, because each is a standing property of the formula
  rather than a fact about this year:

  - the **size cliff** in targeted assistance's capacity tier — nothing below 200 ADM, 5% to
    400, ramping to full at 600 — which binds for five districts today;
  - the wealth tier's **0.8 index floor**, which is not a chosen threshold: its two coefficients
    stand in exactly that ratio, so the bracket reaches zero there;
  - gifted's **floor**, worth $93,993 before the state share, on which 370 districts sit;
  - the **supplemental targeted assistance** tier, which qualifies 36 districts and pays nothing.
    A funded line at zero is not the same as an absent one, and only the node can say so.

  Carried in `crates/project::panel` and verified in
  `crates/project/tests/the_remaining_categoricals.rs`.

- ~~**Three sheets outside foundation funding are cached and unread.**~~ **CLOSED** — all three
  read and reproduced. What they turned out to hold:

  - the **performance supplement** is $55.7m, not the small line its absence suggested, and it is
    distributed **inversely to need**: $54.74 a pupil in the least-poor fifth of districts against
    $23.31 in the poorest;
  - the **enrollment growth supplement** is a **cliff** — $250 on every pupil at 3% three-year
    growth, and New Lexington missed it by three hundredths of a percentage point and $430,477;
  - **transportation** is $726m, larger than special education, making it the second-largest
    single program in Ohio's school funding. It has **two competing rate bases**, a **50% state
    minimum share** against the formula's 10%, two supplements rewarding opposite things, **its own
    guarantee** on a FY2021 base, and a **proration factor of 0.91746** on its special education
    line — the first parameter in this corpus that encodes a shortfall rather than a rate.

  A fourth line, **preschool special education**, is now read too: $148m, a flat $4,000 a pupil
  plus the six school-age weights at half. With it, **95.00% of what sits between `[H] Foundation
  Funding` and `[R] Total State Support` is named**, leaving a $63.6m residual with no component
  behind it yet. [verified]

  ~~Two of those want corpus nodes of their own, and neither has one yet:~~ **BOTH WRITTEN**,
  along with nodes for every component named above. The reasoning that produced them:

  - **A second temporary transitional aid guarantee.** `formula-component` holds one node by that
    name, for the guarantee on foundation funding. Transportation has another, on a different base
    year and a different base, holding 38 districts. Whether these are two instances of one
    mechanism or two mechanisms sharing a name is the same modelling question the categoricals
    posed, and it should be answered before either node is edited. [open]

    **THERE ARE THREE, NOT TWO, AND THE EXISTING NODE DESCRIBES ITS OWN MECHANISM INCOMPLETELY.**
    Reading `Detail_SFPR`'s formulas closed the last $63.6m of the gap and turned up two things
    the node does not have:

    1. **`[K] Formula Transition Supplement`** is a second hold-harmless stacked on the first,
       against a *larger* FY2021 base — `[L1]`, which includes transportation where `[H2]` does
       not. $63.6m to **144 districts, 17 of which draw nothing from the guarantee**. That is the
       whole of the residual, so nothing material is left unnamed between `[H]` and `[R]`.
    2. **The guarantee has an open-enrolment clawback.** `[I]` is `funding base − [I1] − foundation
       funding`, and `[I1]` reduces the guarantee of a district whose open enrolment FTE fell by
       more than `max(10% of last year, 20 FTE)` — at **$8,241.61 an FTE**, the statewide average
       base cost per pupil, at full value rather than the district's state share. 43 districts,
       $5.1m withheld; Columbus lost 106.2 FTE and $674,561.

    The second is the one that matters for the node. A guarantee described without the clawback
    reproduces correctly for 566 districts and wrongly for 43 — few enough to read as rounding,
    which is why it survived this long. [verified —
    `crates/project/tests/the_supplements_outside_the_formula.rs`]

    So the answer to the modelling question is **three nodes**, on the same grounds as the six
    categoricals: three different FY2021 bases, three different sets of districts, and none nested
    in another. And the existing node needs the clawback written into it before anything else is
    built on it.
  - **Proration as a parameter class.** The corpus holds rates, weights, prices, thresholds and
    blends. A proration is none of those: it is an appropriation divided by an entitlement, so it
    is a fact about a budget rather than about a formula, and it moves for reasons nothing else in
    `parameter/` moves for. It also cannot be recovered from a published amount. [open]

    **Preschool special education settles what the class would hold.** Its sheet carries the
    **appropriation limit** — $147,500,000 — in a cell beside the factor, which is the only place
    in the calculator that shows a proration's two halves at once. A proration node therefore wants
    three properties and not one: the factor, the appropriation it was set against, and the
    entitlement it was divided into.

    It also shows why the class matters. **At the stated factor of 0.96854448 the program totals
    $148,408,184 — $908,184 over its own limit.** A third cell states $146,708,228.07, matching
    neither. The likeliest reading is a factor calibrated against an earlier ADM vintage and never
    recalibrated after the counts were refreshed; the calculator is a projection published before
    the fiscal year, so recalibration before payment is expected. But a parameter that has silently
    stopped satisfying its own constraint is exactly what a node with all three properties would
    catch and a node with only the factor would not. [verified —
    `crates/project/tests/the_supplements_outside_the_formula.rs`]

    Three prorations are now known: transportation's special education line at 0.91746,
    transportation's general factor at 1.0, and this one. Only this one publishes its
    appropriation.

  The original entry:

- **Three sheets outside foundation funding are cached and unread.** The corpus already names the
  gap between `[H] Foundation Funding` and `[R] Total State Support` — transportation, preschool
  special education, special education transportation, the performance supplement — and carries
  the difference as a number. The FY2027 calculator computes each of them on its own sheet, and
  none has been opened:

  - **`Performance Supplement`.** $13 per enrolled pupil, gated on `O1 Overall Performance Rating
    Stars` and two years of progress component ratings, with an explicit `Q4. Overall Eligibility`
    column. This is the one place Ohio's funding formula **pays on report-card outcomes**, and
    the corpus has an `outcome` block per district already, so the two can be joined. A component
    that conditions money on a rating deserves a node whatever its size.
  - **`Transportation`.** Two statewide rates — $1,337.175 per rider and $6.867 per mile — with a
    **1.5 weight for community-school and STEM riders and 2.0 for non-public**, a 50% state
    minimum share, and a **special education transportation proration factor of 0.91746**. The
    proration is the interesting number: a factor below one means the appropriation did not cover
    the computed entitlement and every district's amount was scaled down to fit. That is a
    different kind of parameter from a rate and the corpus holds nothing like it yet.
  - **`Base_Enrollment Growth`.** A $40 per-pupil base funding supplement, plus a **$250 per-pupil
    enrollment growth supplement** gated on `M1 Enrollment Change Percentage` against a threshold.
    Worth reading beside the guarantee, which pays districts for enrolment they have **lost**:
    the same formula pays a premium in both directions and the two supplements have never been
    looked at together.

  All three sit in `.cache/sources/fy27-calculator.xlsx` and need no new connector. The workbook
  carries its own formulas, so the mechanisms are recoverable directly rather than by inference —
  see how the four remaining categoricals were settled in `the-last-four`.

