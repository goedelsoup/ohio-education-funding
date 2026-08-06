# litigation — actions

## Queries

- **Precedent chain.** Follow `cites` to reconstruct how a holding was built on or narrowed by
  later decisions.
- **Legislative response.** Follow `prompts` to the enactments that answered a decision, and
  compare the decision's stated defect against what the enactment actually changed. The gap
  between the two is a recurring finding in this domain.
- **Doctrine application.** For a doctrine, retrieve every case that invoked it and compare
  outcomes. *Walter* and *DeRolph I* applied the same constitutional text to similar facts and
  reached opposite results eighteen years apart.
- **Party exposure.** Follow `has-party` to the agencies involved, then pull their funding
  series for the years around the decision.

## Transitions

- **Appeal or reconsideration.** A later decision in the same action becomes a new node with a
  `cites` edge, never an edit to the earlier one.
- **Remedy termination.** A case's enforcement ends without compliance. Recorded in `remedy`;
  this is the *DeRolph* case and it needs saying explicitly wherever it appears.

## Connectors

- `ohio-courts` — opinions and citations.
- `ohio-laws` — the enactments that follow, for the `prompts` edge.

## Skills

- `provenance-trace` — resolves a holding to its published opinion.
