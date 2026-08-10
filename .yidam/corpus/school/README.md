# school

A single school building. The class exists because Ohio's accountability system does not work at
the grain its funding system does: the formula pays agencies, ESSA identifies buildings, and
R.C. 3302.12 attaches its intervention to a building. Rolling that up to the district would record
the consequence and discard the object that triggered it.

**Exemplars, not the panel.** Ohio has 3,318 rated buildings and they live in
[`report-card-2425-buildings.csv`](../../../crates/dispersion/fixtures/report-card-2425-buildings.csv).
What belongs here is the handful that span the distinctions that matter — the same rule
[education-agency](../education-agency/) follows.

See the class definition: [school.ont.yml](../school.ont.yml).

## Instances

| Node | District | PI 2024-25 | Chronic absenteeism | Why it is here |
|------|----------|-----------:|--------------------:|----------------|
| [sheridan-high-school](sheridan-high-school.yml) | Northern Local | 89.9 | 25.9% | Nathan DeRolph's school; the building the litigation is named for |
| [anton-grdina](anton-grdina.yml) | Cleveland Municipal | 36.8 | 66.7% | The bottom of the distribution, in the district that had a distress commission |
| [barrington-road-elementary](barrington-road-elementary.yml) | Upper Arlington | 105.9 | 2.5% | The contrast, in the corpus's high-wealth exemplar district |

**Sixty-nine points of Performance Index and sixty-four points of chronic absenteeism**, between
two buildings in the same state measured the same way in the same year. All three are flat across
2022-23, 2023-24 and 2024-25 — the span both the academic distress trigger and the ATSI-to-CSI
escalation read.

## What is not held, and it is the important part

**The identification lists are now held** — 231 CSI, 117 ATSI, 60 TSI, in
[`identified-schools-2026.csv`](../../../crates/dispersion/fixtures/identified-schools-2026.csv).
Sheridan is CSI by escalation from ATSI; Anton Grdina and Barrington Road are on no list. What is
*not* held is a Title I service flag, without which an absence cannot be read: CSI selects among
Title I served schools, so a building can perform badly and never appear. [open]

**And the lists have no history.** The department republishes each in place under a dated filename
rather than archiving cycles, so a school that exited before this file was written looks like one
never identified. Some CSI rows still carry a 2018 identification against exit criteria that allow
three years. [verified]

**No overall rating.** R.C. 3302.10 and CSI both key on the report card's *overall* rating, and
neither published building file carries it — `Achievement_Building` gives the achievement
component and `Building_Details` gives none. What is held is an input to the trigger, not the
trigger. [unentered]

**No grade spans, and no commission timeline**, so the Cleveland building cannot yet be placed
against the years its district was under state control. [open]
