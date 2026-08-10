# Ohio's Consolidated State Plan under ESSA, and the April 2026 School Improvement amendment

**Source.** Ohio Department of Education and Workforce, *Ohio's Consolidated State Plan* submitted
to the U.S. Department of Education under the Every Student Succeeds Act; and *Ohio Amendments to
Approved ESSA Plan*, April 2026, a redlined amendment request covering School Improvement from
page 43 of the approved plan.
**Type.** Primary source — the state's own plan as submitted to a federal agency.
**Location.** `education.ohio.gov/Topics/Every-Student-Succeeds-Act-ESSA` resolves. The April 2026
amendment PDF was **supplied to this repository directly**; its canonical URL was not located, and
a guessed `getattachment` path under the topic page returns 404.

**Status.** *Not retrieved, not committed, not digest-pinned.* No connector fetches it and nothing
in `crates/` reads it. Everything below is read from the supplied document, which is why the
claims resting on it are tagged as claims about what the document says rather than about what is
in force.

**And the amendment is a request.** It states that it "is coordinated with Ohio's Waiver request",
and nothing in it evidences approval by the U.S. Department of Education. A reader must not treat
the redlined provisions as operative.

## What it contains

The methodology by which Ohio identifies schools for federal improvement status, the criteria for
leaving it, and what the state does to schools that do not:

- **CSI** — Comprehensive Support and Improvement. The lowest-performing 5% of Title I served
  schools on the Ohio School Report Card overall rating; for schools with fewer than three rated
  components, the lowest 5% by chronic absenteeism; public high schools with a federal graduation
  rate at or below 67%; and schools that have not exited ATSI within three years.
- **TSI** — one or more subgroups consistently in the bottom 2% of their subgroup rank order, with
  a 2.5-star-or-lower subgroup rating.
- **ATSI** — one or more subgroups at or below the CSI identification threshold score.

Identification is building-level, on a numerical overall rating that is rank-ordered. CSI and ATSI
are identified every three years; the next CSI cycle is 2027-2028.

## Why a funding corpus holds it at all

**Because two of its provisions are fiscal.** Among the more rigorous interventions proposed for
CSI schools that miss exit criteria within three years is *"annual approval of expenditure of all
federal funds by the Department including but not limited to Title IA, IDEA, Noncompetitive School
Improvement Funds (1003)"*, and the amendment turns §f — previously "Not applicable" — into a
district-level regime whose last item is *"approval by the Department of expenditure plans for
federal funds including Title I A, Title II, Title III, Title IV, and IDEA funds."*

That is a state gate on federal money, conditioned on academic performance. The corpus's
[Title I](../corpus/revenue-stream/title-i.yml) and
[IDEA Part B](../corpus/revenue-stream/idea-part-b.yml) nodes described those channels'
restrictions in terms of supplement-not-supplant and said nothing about it.

## Caveats

- **Mostly out of scope, and the proportion matters.** This is an accountability document. The
  overwhelming majority of it — identification methodology, exit criteria, technical assistance,
  evidence tiers — bears on a domain this corpus does not model. Only the expenditure-approval
  provisions are funding facts.
- **The grain does not match.** CSI, TSI and ATSI attach to *buildings*. This corpus's level of
  analysis is the education agency in a fiscal period, and nothing here joins to it without a
  building-to-district roll-up the repository does not have.
- **The same rating pays and punishes.** The Ohio School Report Card overall rating drives
  [the FSFP performance supplement](../corpus/formula-component/fsfp-performance-supplement.yml),
  which is money, and CSI/ATSI identification, which is sanction. One measure, two functions,
  and the corpus currently models only the first.
- **Three of the proposed interventions are transfers out of the district sector.** Conversion to
  a charter school operated by an approved school management organization, merger with a school
  operated by one, and closure. Those are funding events routed through the community-school
  channel the corpus cannot model at all — the same hole the `deduction` calculator stub records.
- **Most of the redline is tense.** Roughly a third of the marked changes convert `will be` to
  `are` or `were`, rewriting 2017 aspiration into present tense. Reading the red as change
  overstates what moved.
- **The direction of the substantive changes is one-way.** Three insertions of "at least" before
  the 5% threshold plus a new sentence permitting identification beyond it, alongside a
  sixteen-item intervention ladder that did not previously exist. Wider net, harder consequences.

## Used by

- [`revenue-stream/title-i`](../corpus/revenue-stream/title-i.yml)
- [`revenue-stream/idea-part-b`](../corpus/revenue-stream/idea-part-b.yml)
