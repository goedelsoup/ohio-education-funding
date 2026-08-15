# program — actions

## Queries

- **Net position.** For an agency and fiscal period, sum formula aid, program diversions, and
  program receipts. A foundation payment alone is not a district's state funding position, and
  reporting it as one is the most common error in this domain.
- **Diversion series.** For a district, the amount and student count leaving via each
  scholarship program by year — the `deduction` calculator's output.
- **Eligibility drift.** Follow `eligibility` history to see how a program's population changed
  without the program being renamed. EdChoice moved from a targeted intervention to universal
  eligibility while keeping its name.
- **Capital position.** For a district, facilities assistance received, local share required,
  and whether the local share required a bond issue.
- **What has this node been wrong about?** Read `revisions:`. Each entry carries the claim as
  it stood, what replaced it, and the test or source that settled it — so a withdrawn figure can be
  recognised rather than re-derived, and `found_by` gives the check to re-run.

## Transitions

- **Eligibility expansion.** Recorded in `eligibility` with the enacting bill; the node is not
  duplicated unless the mechanism itself changes.
- **Mechanism change.** A program moves from deduction to direct appropriation, as community
  school funding did under the Fair School Funding Plan. This is a `mechanism` change and
  warrants explicit note, because it silently alters every affected district's reported
  revenue.
- **Invalidation.** A court holds the program unconstitutional. Recorded via the incoming
  `challenges` edge; the program node continues to describe the program as it operated.

## Calculators

- `deduction` — per-agency diversion by program and year.

## Connectors

- `dew-foundation` — scholarship counts and amounts by resident district.
- `ofcc-projects` — facilities projects, state share, local share.
- `lsc-budget` — appropriation lines.
