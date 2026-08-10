# legislation

The enacted instruments that built Ohio's school funding system — General Assembly bills,
almost always biennial budget acts, plus the constitutional provision the whole system is
measured against. Ohio does not legislate school funding as standalone policy; the formula
lives inside the operating budget, which is why the nodes here are mostly bill numbers that
also carry Medicaid, corrections, and highway appropriations. Reading a funding change out of
one of these acts means reading a few dozen sections out of several thousand pages.

Constitutional provisions sit in this class rather than in one of their own because they
occupy the same structural position: an authority that establishes, constrains, and gets
litigated against. Article VI, Section 2 is the target of every case in
[`litigation/`](../litigation/), and treating it as a separate class would have duplicated
every edge that already runs to legislation.

See the class definition: [legislation.ont.yml](../legislation.ont.yml).

## Instances

| Node | Year | What it did |
|------|------|-------------|
| [ohio-constitution-article-vi-section-2](ohio-constitution-article-vi-section-2.yml) | 1851 | Imposed the thorough-and-efficient duty |
| [hb-920-1976](hb-920-1976.yml) | 1976 | Froze the dollar yield of voted millage against inflation |
| [hb-94-2001](hb-94-2001.yml) | 2001 | Rebuilt the formula after DeRolph II; parity aid, gap aid, six special education weights |
| [hb-66-2005](hb-66-2005.yml) | 2005 | Phased out the tangible personal property tax |
| [hb-1-2009](hb-1-2009.yml) | 2009 | Enacted the Evidence-Based Model |
| [hb-153-2011](hb-153-2011.yml) | 2011 | Repealed the EBM; installed the Bridge formula |
| [hb-110-2021](hb-110-2021.yml) | 2021 | Enacted the Fair School Funding Plan |
| [hb-33-2023](hb-33-2023.yml) | 2023 | Refreshed cost inputs to FY2022; made EdChoice universal |
| [hb-96-2025](hb-96-2025.yml) | 2025 | Phase-in to 100%, cost inputs held at FY2022 |

Read the last three in sequence and the argument of the current era is visible: H.B. 110 built
a formula that prices staffing at a reference year, H.B. 33 proved the reference year can be
refreshed, and H.B. 96 declined to refresh it while completing the phase-in. Freezing was a
choice against an established practice, not an oversight.

## Known gaps

**[H.B. 583](hb-583-2022.yml) now has a node**, though what it changed does not: LSC's final
analysis for it is a separate document `lsc-budget` does not retrieve, so the corpus can say the
corrections are operative and not which provisions moved. [open] Beyond it, the class thins out
badly before 2005: the enactments that built and repeatedly amended the foundation base cost
formula between 1976 and 2005 are represented by H.B. 920 and now
[H.B. 94](hb-94-2001.yml), so the corpus can say what that regime did but still not which
General Assembly did most of it. [open]

**And the pre-2005 nodes cannot be sourced the way the later ones are.** `ohio-laws` serves the
Revised Code as it stands today, which is the wrong document for an act from 2001. Every section
H.B. 94 is cited by has since been repealed or reused — R.C. 3317.012 now reads "joint vocational
school district base cost", R.C. 3317.0217 now reads "targeted assistance funding" — so following
a contemporary citation to the current code silently lands on a different programme. Nodes for
this era rest on secondary recitation until the session laws are retrievable, which is the same
gap that keeps `lsc-budget` from carrying the pre-2000 record. [open]
