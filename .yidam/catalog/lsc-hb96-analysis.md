---
used-by:
  - ../corpus/draft-legislation/hb-96-with-refreshed-inputs.yml
  - ../corpus/formula-component/fsfp-community-school-equity-supplement.yml
  - ../corpus/formula-component/fsfp-jvsd-state-share-of-base-cost.yml
  - ../corpus/legislation/hb-1-2009.yml
  - ../corpus/legislation/hb-110-2021.yml
  - ../corpus/legislation/hb-119-2007.yml
  - ../corpus/legislation/hb-153-2011.yml
  - ../corpus/legislation/hb-166-2019.yml
  - ../corpus/legislation/hb-583-2022.yml
  - ../corpus/legislation/hb-64-2015.yml
  - ../corpus/legislation/hb-66-2005.yml
  - ../corpus/legislation/hb-94-2001.yml
  - ../corpus/legislation/hb-95-2003.yml
  - ../corpus/legislation/hb-96-2025.yml
  - ../corpus/parameter/enrolment-supplement-amounts.yml
  - ../corpus/parameter/fsfp-phase-in-percentage.yml
  - ../corpus/parameter/performance-supplement-rate.yml
---
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
extraction phase, and the enacted-Greenbook figures could not then be taken from it directly.
That is handled now: `connect` retrieves the department's Greenbook as `hb96-edu-greenbook`,
pins it in `source-digests.txt`, and commits the extract as
[`dew-greenbook.txt`](../../crates/project/fixtures/dew-greenbook.txt), so enacted figures drawn
from it are `[verified]` rather than inference.

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

## Every act's enrolled analysis dates it, on its cover

The bill analysis *as enrolled* opens with an `Effective date:` line, and for a budget act that
line is the only place outside the enrolled act's own section-by-section clauses that says which
half took effect when. The enrolled PDF itself cannot: it prints the governor's approval and the
Secretary of State's filing stamp as blank templates.

The same path shape serves it for older acts, with `en` for `en0` and `enrolled` for `enacted`:
`lsc.ohio.gov/assets/legislation/{GA}/{bill}/en/files/{bill}-bill-analysis-as-enrolled-{GA}{st|nd|rd|th}-general-assembly.pdf`.
The ordinal suffix is part of the name — `131st`, `133rd` — and a wrong one is a 404. Read 29
September 2026 for their covers alone, and not fetched by the connector:

| act | cover |
|---|---|
| H.B. 94, 124th | certain provisions effective September 5, 2001 |
| H.B. 66, 126th | June 30, 2005; certain provisions September 29, 2005 |
| H.B. 119, 127th | June 30, 2007; certain provisions September 29, 2007 |
| H.B. 1, 128th | July 17, 2009; certain provisions October 16, 2009 |
| H.B. 153, 129th | June 30, 2011; certain provisions September 29, 2011 |
| H.B. 64, 131st | June 30, 2015; certain provisions September 29, 2015 |
| H.B. 166, 133rd | July 18, 2019; most provisions October 17, 2019 |
| H.B. 110, 134th | operating appropriations June 30, 2021; other provisions generally September 30, 2021 |

H.B. 95 of the 125th has no analysis at that path: the URL answers with LSC's *Selected Issues*
fiscal volume instead, which carries no effective date. Its dates, and H.B. 583's, come from the
legislature's version index, recorded at [`ohio-bills`](ohio-bills.md), which agrees with every
cover above where it has a value.

## Feeds connector

[`lsc-budget`](../../crates/connect/sources/lsc-budget.md)
