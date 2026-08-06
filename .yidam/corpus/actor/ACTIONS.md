# actor — actions

## Queries

- **Who set this?** Given a `parameter` or `funding-regime`, walk back through `legislation`
  to the enacting actor and the General Assembly that passed it.
- **Succession chain.** Follow `succeeds` backward from a current agency to reconstruct which
  body held an authority in a given fiscal period — required before attributing a
  publication to a publisher.
- **Publication provenance.** Given a numeric claim, resolve which actor published the
  underlying series and whether an alternative actor publishes a competing figure for the
  same quantity.

## Transitions

- **Succession.** An actor is superseded by another. The superseded node is never deleted;
  a `succeeds` edge is added to the successor and the predecessor's `authority` is closed out
  with an end date.
- **Authority change.** A statute expands or removes an actor's power without creating a new
  entity. Recorded as a revision to `authority`, linked to the enacting `legislation`.

## Skills

- `provenance-trace` — resolves a numeric claim to the publishing actor and its catalog entry.

## Connectors

- `ohio-laws` and `ohio-courts` are keyed on actor: bills resolve to the General Assembly,
  opinions to the Supreme Court of Ohio.
