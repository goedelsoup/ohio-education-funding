# nces-ccd — connector (stub)

**Source.** National Center for Education Statistics, Common Core of Data: agency identifiers,
enrollment, agency type and status, and the year-over-year identifier changes that record
mergers, splits, and closures.

**Feeds.** [`education-agency`](../../.yidam/corpus/education-agency/).

## Retrieval interface

```
fetch_agencies(fiscal_year, state)   -> Vec<AgencyRecord>
fetch_enrollment(fiscal_year, leaid) -> EnrollmentRecord
identifier_changes(from_year, to_year) -> Vec<AgencyChange>   // merge | split | close | open
```

`identifier_changes` is the reason this connector is approved despite overlapping with
`dew-foundation` on enrollment. A corpus spanning 1851 to the present is a panel whose members
change, and a long series assembled without accounting for consolidation is silently wrong —
it is also what populates the
[`merged-into`](../../.yidam/corpus/education-agency.ont.yml) edge.

## Constraints

- Keys on federal LEAID. IRN crosswalk shared with `census-f33`.
- NCES enrollment and Ohio ADM are different quantities. Neither substitutes for the other in
  a formula computation.
- Coverage begins in the late twentieth century; earlier agency history has no federal source.
- Offline mode required.

## Status

Stub. Approved in [decisions/proposals.yml](../../.yidam/decisions/proposals.yml); not
implemented.
