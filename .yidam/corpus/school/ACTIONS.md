# school — actions

## Queries

- **Building to agency to formula.** Follow `operated-by` to the district, then into the funding
  side. This is the only path in the corpus that reaches from an accountability object to a
  funding one, and it is why the class was added.
- **Three-year read.** The distress trigger and the ATSI escalation both look at three consecutive
  years. The fixture carries 2022-23, 2023-24 and 2024-25 in one row for that reason; a
  single-year claim about a building cannot speak to either.
- **Contrast pairs.** Follow `contrasts-with`. A two-building comparison is an illustration and
  never an estimate — say so wherever one is used.
- **What has this node been wrong about?** Read `revisions:`. Each entry carries the claim as
  it stood, what replaced it, and the test or source that settled it — so a withdrawn figure can be
  recognised rather than re-derived, and `found_by` gives the check to re-run.

## Writing a node

- **Do not present the achievement star as the overall rating.** They are different numbers and
  only the overall one carries consequence. The published building files do not contain it.
- **Chronic absenteeism is a criterion, not colour.** For a building with fewer than three rated
  components it decides CSI identification outright. Record whether the building has enough rated
  components for that to apply, because the same number means different things either side of it.
- **A Performance Index is scored over tests that should have been taken.** A building where a
  large share of students are chronically absent is being scored partly on attendance through a
  measure nominally about learning. Do not read the index as clean where the absenteeism rate is
  high; say that the two cannot be separated here.
- **Status goes in `roles` with its years**, never in the label or the identity.
