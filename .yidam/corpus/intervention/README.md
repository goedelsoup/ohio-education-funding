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

## Why this class is in a funding corpus

Because R.C. 3302.10(D)(j) hands the power of "creating a budget for the district" to a chief
executive officer the department pays. The formula still computes an amount and the board still
levies; what changes is who decides how the money is spent. An accountability mechanism that
reassigns fiscal control is a funding mechanism whatever chapter it sits in.

## Known gaps

**The ESSA intervention ladder is not instantiated.** Ohio's April 2026 amendment proposes sixteen
more rigorous interventions for CSI schools, from revising an improvement plan to closure, and
modelling them needs one node per rung rather than one for the ladder. [open]

**No district series.** Which districts have been under a commission and when is not held; the
corpus names Cleveland's in passing and nothing else. [open]

**H.B. 70 (2015) has no legislation node**, so `authorized-by` runs to the Revised Code section
rather than to the act that produced it — the same act R.C. 3302.10 refers to when it speaks of a
commission "still in existence on October 15, 2015". [open]
