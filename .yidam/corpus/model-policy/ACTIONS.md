# model-policy — actions

## Queries

- **Mechanism comparison.** Read `district_funding_effect` here against the `mechanism` property on
  the Ohio [program](../program/edchoice-expansion.yml) node. This is the class's primary use and
  the only one whose answer is currently established on both sides.
- **Convergence and divergence separately.** Follow `parallels` and `departs-from` to the same
  program node and expect both to be present. Ohio's eligibility design moved toward the model
  while its funding mechanism moved away; a query that reports only one edge reports half of it.
- **What a proposal would cost the count.** Follow `bears-on` to [enrolled ADM](../metric/enrolled-adm.yml)
  before pricing any ESA proposal. The model keeps the student in the resident district's
  enrollment while subtracting the award, and a funding model that missed that clause would
  misstate both the numerator and the denominator.
- **What has this node been wrong about?** Read `revisions:`. Each entry carries the claim as
  it stood, what replaced it, and the test or source that settled it — so a withdrawn figure can be
  recognised rather than re-derived, and `found_by` gives the check to re-run.

## Writing a node

- **Do not write a node for a document that only states a position.** It belongs in the catalog,
  cited from the Ohio node it bears on. A node here requires a mechanism Ohio can be checked
  against — see [README](README.md).
- **`adoption_evidence` is mandatory and starts at `[open]`.** Never infer adoption from
  resemblance, and never leave the field off because the answer is nothing. State the absence and
  say what would settle it.
- **Choose the verb so it survives the provenance being open.** `parallels`, not `template-for`
  or `adopted-as`. The relationship has to remain true if `adoption_evidence` is never filled.
- **Quote the text.** Provisions carry `[verified — ALEC model text]` and are quoted rather than
  paraphrased, because the whole value of the comparison is that both sides are exact. A
  paraphrased mechanism cannot be diffed against a statute.
- **Record the publisher's alignment once, in the catalog record**, and reference it. Repeating it
  in every node's prose turns a provenance fact into editorializing.

## Transitions

- **Provenance established.** `adoption_evidence` moves off `[open]` only on sponsor testimony,
  committee records, or a textual diff against the enacted section. If it ever does, the edge verb
  should be revisited — `parallels` would then be understating what is known.
- **A variant is retrieved.** Model policies come in variants that differ on exactly the fields
  this class records. A new variant is a new node, not an edit, because the publisher versions them
  separately.

## Calculators

None. Nothing in this class produces a number, and nothing here may be used as a funding input —
a model bill's award basis is a proposal, and this repository's calculators read committed Ohio
fixtures only.

## Connectors

None, and none is planned. Model policies are published as web pages and static PDFs by their
authors, retrieved by hand, and versioned by publication date rather than by digest — the same
footing as [`fordham-base-cost-critique`](../../catalog/fordham-base-cost-critique.md). Note that
Heritage's interactive report card returns HTTP 403 to anything identifying itself as a program,
so a connector aimed at it would join the two already
[blocked](../../../crates/connect/README.md).
