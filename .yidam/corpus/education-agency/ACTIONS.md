# education-agency — actions

## Queries

- **Funding history.** For one agency across every regime it lived through, in nominal and
  constant dollars, with each figure attributed to the component that produced it. The
  `agency-profiler` traversal.
- **Floor status.** Whether the agency is at the 20-mill floor in a given fiscal period, which
  determines whether valuation growth reaches its revenue at all.
- **Net state position.** Foundation aid plus program receipts minus program diversions. Never
  report a foundation payment alone as an agency's state funding.
- **Peer comparison.** Retrieve `peer-of` agencies and compare per-pupil resources against
  valuation per pupil — the wealth-neutrality test at the case level.
- **Panel continuity.** Follow `merged-into` before assembling any long series. District
  consolidations break longitudinal comparison silently.
- **What has this node been wrong about?** Read `revisions:`. Each entry carries the claim as
  it stood, what replaced it, and the test or source that settled it — so a withdrawn figure can be
  recognised rather than re-derived, and `found_by` gives the check to re-run.

## Transitions

- **Merger or dissolution.** The agency ceases and its territory joins another. A
  `merged-into` edge is added; the node is retained so historical series remain attributable.
- **Closure.** A community school ceases operation. The node remains — ECOT's funding history
  is the reason it is in this corpus.
- **Role change.** The agency enters or leaves the 20-mill floor, a guarantee, or academic
  distress status. Recorded in `roles` with the fiscal periods, never in identity.

## Calculators

- `millage`, `local-capacity`, `deduction`, `foundation`, `deflate`.

## Connectors

- `dew-foundation`, `tax-abstract`, `nces-ccd` (identifier crosswalks), `census-f33`.
