# LSC Budget Analysis — H.B. 96 (FY2026-27)

**Source.** Ohio Legislative Service Commission, Legislative Budget Office. The document set
for a single budget act: *Redbook* (as introduced), *Bill Analysis* and *Budget in Brief* at
each stage of passage, *Greenbook* (as enacted), and the appropriation spreadsheet.
**Type.** Primary source — official fiscal and legal analysis.
**Location.** `lsc.ohio.gov/budget/136/main-operating-budget`. The as-enacted Greenbook for
the Department of Education and Workforce is at
`lsc.ohio.gov/assets/legislation/136/hb96/en0/files/hb96-edu-greenbook-as-enacted-136th-general-assembly.pdf`.

**What it contains.** For H.B. 96: the enacted phase-in percentages, the appropriation levels
by line item and fiscal year, and the analysis of every change to the Fair School Funding
Plan. LSC publishes the equivalent set for every biennium, which makes this the only
continuous appropriation series across the whole period this corpus covers, and the main
source for the pre-2000 record where department files do not reach.

**Access constraints.** Freely available. **The `lsc.ohio.gov` asset host does not present a
valid TLS chain to standard fetching tools** — retrieval failed repeatedly during the first
extraction phase, and the enacted-Greenbook figures below could not be taken from it directly.
Any pipeline built on this source must handle that, and until it does, figures attributable to
LSC in this corpus rest on secondary reporting and are tagged as inference rather than
verified.

**Caveat — the most important one in the catalog.** LSC publishes at every stage of passage,
and the versions differ materially. The House-passed *Budget in Brief* describes bridge-formula
funding of $11.24 billion in FY2026 and $11.49 billion in FY2027; that is the House proposal,
not the enacted law. A figure from this source is meaningless without the stage it belongs to —
`as introduced`, `as passed by the House`, `as passed by the Senate`, or `as enacted`. The URL
path segment (`in`, `ph`, `ps`, `en0`) encodes it, and every record drawn from here must carry
it.

Separately, LSC simulations are estimates made before a fiscal year closes. They describe the
same quantity as a department payment report and routinely disagree with it. Never merge the
two series.

## Used by

- [`legislation/hb-96-2025`](../corpus/legislation/hb-96-2025.yml)
- [`parameter/fsfp-phase-in-percentage`](../corpus/parameter/fsfp-phase-in-percentage.yml)
- [`fiscal-period/fy2026-27`](../corpus/fiscal-period/fy2026-27.yml)

## Feeds connector

[`lsc-budget`](../../crates/connect/sources/lsc-budget.md)
