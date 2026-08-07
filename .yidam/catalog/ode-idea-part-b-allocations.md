# IDEA Part B Allocations to Districts

**Source.** Ohio Department of Education (now DEW), Special Education Data and Funding —
annual IDEA Part B special education allocation tables, one per fiscal year.
**Type.** Primary source — official allocation record.
**Location.** `education.ohio.gov`, Topics → Special Education → Special Education Data and
Funding → Special Education Part B Allocations. The FY2021 file is
`Fiscal-Year-2021-IDEA-Part-B-Special-Education-funds-allocation-to-districts.pdf`.

**What it contains.** One row per receiving entity: IRN, entity name, count of public school
students with disabilities, count of non-public students with disabilities, the FY allocation
amount, and the proportionate share amount owed for non-public students. The FY2021 file lists
**979 entities** — traditional districts, community schools, and county boards of developmental
disabilities.

Used here as the corpus's first source of IRNs and its first per-agency numbers of any kind.

**Access constraints.** Freely available. The PDF does not extract through standard text
conversion; `pdftotext -layout` reads it cleanly and preserves the column structure.

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
- **Small counts are suppressed**, appearing as "<10". Any aggregation must decide how to treat
  suppressed cells rather than reading them as zero.
- **Name collisions are real.** "Northern Local" (049056) and "Hardin Northern Local" (047498)
  are different districts. The file carries no county column, so IRN-to-county attribution needs
  a separate crosswalk.

## Used by

- [`education-agency/northern-local-perry`](../corpus/education-agency/northern-local-perry.yml)
- [`education-agency/upper-arlington-city`](../corpus/education-agency/upper-arlington-city.yml)
- [`education-agency/cleveland-municipal`](../corpus/education-agency/cleveland-municipal.yml)
- [`education-agency/eastland-fairfield-ctc`](../corpus/education-agency/eastland-fairfield-ctc.yml)

## Feeds connector

[`dew-foundation`](../../crates/connect/sources/dew-foundation.md), and the IRN crosswalk
[`nces-ccd`](../../crates/connect/sources/nces-ccd.md) depends on.
