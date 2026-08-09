# formula-component — actions

## Queries

- **Component decomposition.** Given an agency-year payment, attribute each dollar to the
  component that produced it. This is the `formula-walk` skill's core operation.
- **Cross-regime lineage.** Follow `replaces` to find the predecessor of a component and
  compare the two mechanisms directly rather than comparing regime totals.
- **Parameter dependency.** List every `parameter` a component consumes, to determine what a
  proposed change would actually touch.

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

  Two of those want corpus nodes of their own, and neither has one yet:

  - **A second temporary transitional aid guarantee.** `formula-component` holds one node by that
    name, for the guarantee on foundation funding. Transportation has another, on a different base
    year and a different base, holding 38 districts. Whether these are two instances of one
    mechanism or two mechanisms sharing a name is the same modelling question the categoricals
    posed, and it should be answered before either node is edited. [open]
  - **Proration as a parameter class.** The corpus holds rates, weights, prices, thresholds and
    blends. A proration is none of those: it is an appropriation divided by an entitlement, so it
    is a fact about a budget rather than about a formula, and it moves for reasons nothing else in
    `parameter/` moves for. It also cannot be recovered from a published amount. [open]

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

