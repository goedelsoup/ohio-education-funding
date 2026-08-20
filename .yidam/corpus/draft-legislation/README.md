# draft-legislation

Provisions that are not law. A bill before the General Assembly, an alternative version of an
act that did pass, or a change nobody has introduced — held here so that "what would this do to
my district?" can be asked of a proposal with the same machinery that answers it for an
enactment.

This class is separate from [`legislation/`](../legislation/) for the reason that class gives
about itself: it is a Kind whose identity is fixed *at enactment*, carrying a signing date, an
effective date and a veto list. A draft has none of those and may never have any. The same split
was already made one document earlier for [`model-policy/`](../model-policy/), whose class
definition states the hazard plainly — holding both in one class would let a reader take a
template for a statute. A proposal is closer to a statute than a template is, so the hazard is
larger here, not smaller.

See the class definition: [draft-legislation.ont.yml](../draft-legislation.ont.yml).

## The label says what the node is, because the page cannot rely on the reader looking

A node's `label` is what appears under the `h1` and in every social card, and a draft rendered
with a bill number and a list of sections reads exactly like an enactment. So the label carries
the provenance rather than leaving it to the `status` property four rows down a table:

| Provenance | Label form | Example |
|---|---|---|
| `counterfactual` | `Counterfactual:` prefix | Counterfactual: H.B. 96 with FY2024 Cost Inputs |
| `introduced` | designation, with the stage | H.B. 643 (136th G.A., as introduced) |
| `hypothetical` | no bill number at all | Refresh on a Rolling Three-Year Reference |

A hypothetical draft may not take a designation even a vacant one. There is no such bill, and a
number is the one thing a reader will believe without checking.

## Instances

| Node | Provenance | Question | Priced |
|------|------------|----------|--------|
| [hb-96-with-refreshed-inputs](hb-96-with-refreshed-inputs.yml) | counterfactual | What would H.B. 96 have paid had it refreshed the cost inputs it froze? | 1 of 1 |
| [fund-the-plan-and-retire-the-guarantee](fund-the-plan-and-retire-the-guarantee.yml) | hypothetical | What does a bill look like that does what both sides ask for, and how much of it can be priced? | 2 of 5 |
| [hb-643-136-introduced](hb-643-136-introduced.yml) | introduced | What does a real pending bill look like when it reaches a model built around the foundation formula? | 0 of 1 |

The first draft is a counterfactual rather than an invention, and deliberately: the enacted act
declined to do something an act two years earlier had done, so the alternative is a real road
not taken rather than a construction. [H.B. 33](../legislation/hb-33-2023.yml) refreshed the
reference year from FY2018 to FY2022; [H.B. 96](../legislation/hb-96-2025.yml) held it at FY2022
while taking the phase-in to 100%. The counterfactual is what the second would have paid on the
first's practice.

Its single provision is already run — see
[fsfp-input-year-refresh](../scenario/fsfp-input-year-refresh.yml) — which makes the first node
in this class a test of the class rather than of the calculators.

**The second exists because the first is the least representative case available.** One provision
binding one lever prices completely, which is the property a real bill never has. So the
hypothetical carries five provisions of which two price, and it establishes the two things the
counterfactual could not:

- **Clauses cannot be priced independently and added.** Its two priced provisions say −$219.0M
  apart and −$143.9M together. The $75.1M difference is 52% of the combined figure, and it is the
  guarantee's `max` counted twice: a district the refresh lifts off the floor is no longer
  standing on the floor for the phase-out to lower.
- **Its three unpriced provisions are unpriced for three different reasons** — one unlevered, two
  outside the model altogether — and the distinction is what `unpriced` exists to carry.

**The third is a real bill, and it prices nothing at all.** H.B. 643 of the 136th caps EdChoice
expansion eligibility at $500,000 of federal adjusted gross income. It is current, it is squarely
about school funding, and every provision it has falls in the scholarship channel that is absent
from this workspace rather than merely unlevered — so the honest answer is not a number.

It also found a defect in the machinery built to hold it. A draft nothing prices produces a
combined policy identical to current law, so the arithmetic gives exactly `0.00`, and the first
version of `Priced::cost` would have published that as the bill's cost. `$0.0M` reads as *this
bill is free*, which is the opposite of *this bill was not priced*. `cost()` and `residual()`
return `Option` now, the CLI prints the absence, and the feed writes `null`.

## Running one

```
edfund-project --drafts                      what is here, and how much of each prices
edfund-project --draft <slug>                the combined run, the attribution, and the residual
edfund-project --draft <slug> --json         the same, machine-readable
```

The unpriced block prints unconditionally and there is no flag to suppress it. A `--brief` that
dropped it would recreate exactly the failure the field exists to prevent, and the first person
to want one would be someone quoting the number.

## What a draft may not do

**State a cost without the count of provisions it fails to price.** `unpriced` is mandatory on
every node here, including the ones where it is empty. The runnable surface is the five levers
in [`crates/project/src/policy.rs`](../../../crates/project/src/policy.rs) and nothing else, so
a bill touching special-education weights, transportation, the scholarship deduction or the
capital channel has provisions that are recorded and not costed. Reporting the priced subset as
the bill's cost is the failure this field exists to make impossible.

**Sum its own provisions.** They do not add. Raising base cost costs the state more per point
the higher it goes, because each increment lifts districts off the guarantee onto a formula that
must then pay them — $47.7M per point at +2% against $72.5M at +20%, measured in
[guarantee-phase-out](../scenario/guarantee-phase-out.yml). A draft's total is one combined run;
the per-provision runs attribute it and the residual between them is reported rather than
absorbed.

**Claim a text it has not pinned.** For a counterfactual or a hypothetical there is no external
text and `text_read` says so. For an introduced bill it is load-bearing in a way it is not for an
enrolled act: the text moves under its own URL as the bill is amended, so a node written against
"the bill" names a moving target. And the retrieval trap is on the record — the Legislative
Service Commission's site soft-404s with a 10,835-byte HTML body under a `200`, so a status code
proves nothing about what came back.

## Known gaps

**One introduced bill, and the ratio it gives is zero.** [`ohio-bills`](../../catalog/ohio-bills.md)
retrieves a pending bill's text and pins it; the connector stays at `retrievable` deliberately,
because turning a bill into provisions is reading rather than extraction. What the one node
establishes is a lower bound and not an estimate: a real bill can price at 0 of 1. What the ratio
looks like across bills, and what it looks like for a budget act, needs more than one. [open]

**Nothing here tells a live bill from a dead one.** The General Assembly publishes versions, not a
procedural history, so a bill that was never reported out of committee is indistinguishable from
one still waiting. `status` records what the version index shows and says the rest is open. [open]

**The runnable surface bounds this class more tightly than it bounds `scenario`.** A scenario is
authored around a perturbation the levers can express, because whoever writes it chooses the
question. A draft is authored around provisions somebody else chose. [open]
