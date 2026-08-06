# legislation — actions

## Queries

- **What did this bill change?** Enumerate the regimes established, parameters set, programs
  created, and revenue streams constrained by one enactment — the `statute-tracer` traversal.
- **Who set this value?** Given a `parameter` and a fiscal period, resolve the enactment that
  fixed the value in force.
- **Response chain.** Follow `prompted-by` from an enactment back to the decision it answers,
  and `amends` backward to reconstruct the statutory lineage of a provision.
- **Veto delta.** For budget acts, compare enrolled text against what survived line-item veto.
  Vetoes in Ohio budget acts have repeatedly touched education funding provisions.

## Transitions

- **Amendment.** A later act modifies this one. Recorded as an `amends` edge on the later
  node; the earlier node is not edited.
- **Repeal.** The regime or program it established ends. The `funding-regime` node's
  `effective_to` closes; this node is unchanged.

## Skills

- `parameter-history` — assembles a parameter's value series and names the enactment behind
  each change.
- `provenance-trace` — resolves statutory text to its catalog entry.

## Connectors

- `ohio-laws` — bill text, effective dates, veto messages.
- `lsc-budget` — redbooks and the Catalog of Budget Line Items; the appropriation levels that
  determine whether an enacted formula is actually funded.
