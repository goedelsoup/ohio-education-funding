# accountability-regime

A named era of measuring schools and attaching consequences to the measurement. Deliberately the
sibling of [funding-regime](../funding-regime/), and on the same time axis, because Ohio has been
changing how it pays schools and how it judges them on overlapping but non-coincident schedules —
and the two histories are almost always told apart.

The regime says what is measured and who is identified. The
[intervention](../intervention/) says what then happens.

See the class definition: [accountability-regime.ont.yml](../accountability-regime.ont.yml).

## Instances

| Node | Level | Measure | What it triggers |
|------|-------|---------|------------------|
| [ohio-report-card](ohio-report-card.yml) | state | Overall rating, letter grade and star | Academic distress at three consecutive years; the FSFP performance supplement |
| [essa](essa.yml) | federal | Ohio's own rating, rank-ordered | CSI, TSI and ATSI identification; expenditure-plan approval |

**The federal regime runs on the state's measure.** ESSA identification in Ohio rank-orders the
Ohio School Report Card overall rating rather than any federal metric, so a change to the state
rating propagates into federal identification with no federal act. The two regimes are not
independent and should not be read as a check on one another.

## Known gaps

**No Child Left Behind has no node**, so the class begins in 2017-18 and cannot yet answer what
changed when ESSA replaced it. [open]

**The pre-2022 report card eras are not modelled.** The star scale replaced letter grades for the
2022-23 report cards and both remain in statute; anything before that is absent, which means the
class cannot currently be read against the Bridge formula decade it overlaps. [unentered]
