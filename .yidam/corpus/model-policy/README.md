# model-policy

Template statutes published by national organizations for adoption by state legislatures. Nothing
in this class is law. It exists so that the advocacy literature behind the school choice channel
can be read against Ohio's enacted arrangement without either being mistaken for the other.

The class was added because the corpus had no place to put a document of this kind. `legislation`
means *enacted* — bill designation, General Assembly, signing date, effective date — and a model
bill has none of those. `actor` covers institutions that enact, decide, administer, or publish the
data this corpus is grounded in, which an advocacy organization does not do. So the publisher stays
a property here and a named alignment in its
[catalog record](../../catalog/alec-parental-choice-scholarship-act.md), which is the treatment
[Fordham](../../catalog/fordham-base-cost-critique.md) has always had.

See the class definition: [model-policy.ont.yml](../model-policy.ont.yml).

## Instances

| Node | Publisher | Version | District funding effect |
|------|-----------|---------|-------------------------|
| [parental-choice-scholarship-act](parental-choice-scholarship-act.yml) | ALEC | 2005, am. 2016 | Deduct; state keeps the remainder |
| [education-savings-account-act](education-savings-account-act.yml) | ALEC | 2017, am. 2017 | Deduct; student stays in the district's count |

## What makes a node here worth having

Not influence, which this repository cannot measure. **A model policy earns a node when it states
a mechanism that Ohio's enacted arrangement can be checked against.** Both current nodes qualify on
the same field: each specifies that the award is subtracted from the resident district's state aid,
and this corpus has established — negatively, exhaustively, against the department's own FY2027
calculator — that Ohio's post-FSFP scholarship channel does not work that way. The comparison is
falsifiable in both directions, which is what distinguishes a node from a bibliography entry.

A model policy that only stated a position would belong in the catalog and nowhere else.

## The rule this class is built around

`adoption_evidence` is `[open]` on every node and must stay a declared field even so. No source
retrieved for this corpus connects either text to any Ohio bill; a resemblance between two
published documents is the only thing two published documents establish. A class that could record
the resemblance but had nowhere to record the missing provenance would read as an accusation, which
is why the field is mandatory rather than omitted when the answer is nothing.

## Known gaps

- **The third variant.** ALEC publishes *Parental Choice Scholarship Program Act (Universal
  Eligibility, Means-Tested Scholarship Amount)*, whose title describes Ohio's enacted EdChoice
  expansion exactly — universal eligibility, award scaled by family income. Its text is not
  retrieved. It is the highest-value next read in this class. [open]
- **Heritage publishes no model bills**, so its school choice literature is present as
  [catalog records](../../catalog/heritage-education-freedom-report-card.md) cited from the Ohio
  nodes they bear on, and has no instance here. The asymmetry is real and not an omission.
- **Provenance tooling.** A textual diff of a model against an enacted Ohio section is the one
  method that could move `adoption_evidence` off `[open]`, and
  [`regime-diff`](../../../crates/regime-diff/) already does that for successive Ohio statutes.
  Nothing points it at a model bill yet. [open]
