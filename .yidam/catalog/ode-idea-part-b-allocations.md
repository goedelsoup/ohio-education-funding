---
used-by:
  - ../corpus/education-agency/northern-local-perry.yml
  - ../corpus/education-agency/upper-arlington-city.yml
  - ../corpus/formula-component/fsfp-special-education-weights.yml
  - ../corpus/revenue-stream/idea-part-b.yml
---
# IDEA Part B Allocations to Districts

**Source.** Ohio Department of Education (now DEW), Special Education Data and Funding —
annual IDEA Part B special education allocation tables, one per fiscal year.
**Type.** Primary source — official allocation record.
**Status.** *Retrieved and parsed for FY2021-FY2024.* The connector
[`dew-idea-part-b`](../decisions/idea-part-b-connector.yml) fetches the four final editions and
builds `crates/project/fixtures/idea-part-b-allocations.csv`, one row per agency per year keyed
on IRN. The two district nodes read off the FY2021 list by hand predate it. FY2011-FY2020 are
served as `_Adjusted` editions in a different layout and are not read. Neither is the FY2022
American Rescue Plan supplement, which carries its own proportionate share.
**Location.** `education.ohio.gov`, Topics → Special Education → Special Education Data and
Funding → Special Education Part B Allocations. Every edition is attached under the
`Fiscal-Year-FY-15-IDEA-Part-B-Allocation` folder, whatever year it describes.

**What it contains.** One row per receiving entity: IRN, entity name, count of public school
students with disabilities, count of non-public students with disabilities, the FY allocation
amount, and the proportionate share amount owed for non-public students. The FY2021 file lists
**979 entities** — traditional districts, community schools, and county boards of developmental
disabilities.

Used here as the corpus's first source of IRNs and its first per-agency numbers of any kind.

**Access constraints.** Freely available. `pdftotext -layout` reads each edition and keeps its
columns. A long agency name wraps onto the lines around its IRN, so the fixture keeps the IRN
and drops the name.

**Caveats.**

- **Federal money on a federal formula.** IDEA Part B allocation is not state foundation
  funding and does not follow the Fair School Funding Plan. Counts and dollars here answer
  questions about special education population, not about state aid.
- **Joint vocational school districts are absent.** Federal special education money reaches
  JVSD students through their member districts, so no JVSD appears. A statewide roll-up built
  from this list silently omits career-technical provision — an omission that is undetectable
  from the file itself. This is how
  [`eastland-fairfield-ctc`](../corpus/education-agency/eastland-fairfield-ctc.yml) was
  confirmed as structurally different rather than merely missing.
- **Small counts are suppressed**, as "<10" in FY2021 and "-" in FY2022. Any aggregation must
  decide how to treat suppressed cells rather than reading them as zero.
- **No edition prints a total row**, so a statewide figure is a sum this repository computes.
- **Name collisions are real.** "Northern Local" (049056) and "Hardin Northern Local" (047498)
  are different districts. The file carries no county column, so IRN-to-county attribution needs
  a separate crosswalk.

## Feeds connector

`dew-idea-part-b`, recorded in
[`idea-part-b-connector`](../decisions/idea-part-b-connector.yml).
