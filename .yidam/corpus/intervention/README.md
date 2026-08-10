# intervention

What the state does to an agency or a building that its accountability system has identified as
failing. Programs move money toward a recipient on an eligibility rule; interventions move
*authority* away from one on a performance trigger. The two are opposite in direction, which is
why they are separate classes.

An intervention is an Event: it starts on a trigger, runs with named actors and enumerated
powers, and ends on an exit condition. What belongs on the node is the substance that is a fact
about neither endpoint — the statute permitting it, the threshold compelling it, the powers it
transfers, and what it does to money.

See the class definition: [intervention.ont.yml](../intervention.ont.yml).

## Instances

| Node | Unit | Trigger | Fiscal effect |
|------|------|---------|---------------|
| [academic-distress-commission](academic-distress-commission.yml) | education-agency | Three consecutive years at an overall F or under two stars | A state-appointed CEO, paid by the department, creates the district's budget |
| [more-rigorous-interventions](more-rigorous-interventions.yml) | school | CSI, and no exit within three years | Department approval of federal expenditure; four rungs move the building out of the district sector |
| [lea-level-action](lea-level-action.yml) | education-agency | An LEA with a significant number of identified schools | Department approval of federal expenditure plans across five programmes |

**The state regime reaches further than the federal one, and the pair is the argument for holding
both.** An academic distress commission hands a state appointee the power to create the district's
budget. The federal equivalents condition a subset of federal money. A reader who knew only ESSA
would substantially understate what Ohio can do to a district.

## Why this class is in a funding corpus

Because R.C. 3302.10(D)(j) hands the power of "creating a budget for the district" to a chief
executive officer the department pays. The formula still computes an amount and the board still
levies; what changes is who decides how the money is spent. An accountability mechanism that
reassigns fiscal control is a funding mechanism whatever chapter it sits in.

## Known gaps

**The ladder is one node, not sixteen, and that was a change of view.** It was going to be per
rung. The rungs share a trigger, an authority, a unit and an actor and the state requires "one or
more" from a menu rather than applying them in order — sixteen nodes would have repeated four
identical fields to vary a sentence. The enumeration lives in `powers`.

**The commission districts are now held**, from the department's own page: East Cleveland
(released), Lorain (dissolved by H.B. 33 in 2023) and Youngstown (continuing). All three had
revised improvement plans approved on 3 December 2021. What is still missing is when each was
*established* — the page gives endings and not beginnings — and the linked improvement plans,
which are the only place the benchmarks a commission imposed could be learned. [open]

**Cleveland is not among the three**, and its intervention was under the former R.C. 3302.10.
Ohio has had two commission regimes and the corpus previously ran them together. [inference]

**H.B. 70 (2015) has no node, and the blocker is now named rather than assumed.** The act that
produced the current academic distress commission is not retrievable by anything this repository
can reach: `legislature.ohio.gov` serves the bill page as a JavaScript application with no
analysis link in the static HTML, the LSC asset path for a 131st-General-Assembly final analysis
404s, and the search API returns the bill's record with no analysis document in it. [verified —
probed 2026-08-10] What R.C. 3302.10 itself supplies is oblique but real: it refers to a commission
established under a *former* section 3302.10 and still in existence on 15 October 2015, which
dates the replacement and confirms a prior regime existed. [verified] The node waits for a source
rather than being written from recollection.

**And no intervention here has ever been observed.** Every node in this class describes a
mechanism. Whether any Ohio school has been subjected to a rung of the federal ladder, and which
districts have had commissions and when, is unpublished in everything the corpus holds. The class
models what may be done and not what was. [open]
