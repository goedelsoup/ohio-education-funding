# nces-ccd — connector

**Source.** National Center for Education Statistics, Common Core of Data: agency identifiers,
enrollment, agency type and status, and the year-over-year identifier changes that record
mergers, splits, and closures.

**Feeds.** [`education-agency`](../../../.yidam/corpus/education-agency/).

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
[`merged-into`](../../../.yidam/corpus/education-agency.ont.yml) edge.

## Constraints

- Keys on federal LEAID. IRN crosswalk shared with `census-f33`.
- NCES enrollment and Ohio ADM are different quantities. Neither substitutes for the other in
  a formula computation.
- Coverage begins in the late twentieth century; earlier agency history has no federal source.
- Offline mode required.

## Status

**Wired**, for thirty consecutive school years of the LEA universe directory, 1994-95 through
2023-24 — 30,655 Ohio agency-years in
[`ccd-lea-directory.csv`](../../dispersion/fixtures/ccd-lea-directory.csv). See
[`the-directory-cannot-say-why`](../../../.yidam/decisions/the-directory-cannot-say-why.yml) for
the delimited era and
[`before-there-were-service-centers`](../../../.yidam/decisions/before-there-were-service-centers.yml)
for the fixed-width one.

The blocker above said the files "are per-year zips whose column sets change". That is true and it
was never the obstacle it was written as: the columns are resolved by name where there is a header
and by a thirteen-line position table where there is not, and four of the five columns this reader
takes do not move at all across the fixed-width era.

**`identifier_changes` is the part that stays unbuilt, and it is unbuildable from this source.**
Not because the derivation is hard — the diff is a few lines and this repository computes it — but
because the source states the answer and states it wrongly. The CCD has a status code for a
consolidation, code 5, and **Ohio has filed it zero times in 30,655 agency-years**. What Ohio files
for all 689 departures, without one exception in thirty years, is code 2: *"closed with no effect
on another agency's boundaries."* It files the mirror image, code 3, for both of the school
districts it forms. So a `merge | split | close | open` classification derived from here would be
`close` and `open` in every case, which is what the file says and not what happened.

The reason a district went has to be fetched one document at a time. Where those documents are, and
what they cost, is in
[`the-order-was-never-the-states`](../../../.yidam/decisions/the-order-was-never-the-states.yml)
and [`auditor-district-audits`](../../../.yidam/catalog/auditor-district-audits.md).

**Nine older years exist and are not held.** The survey reaches back to 1986-87 in the same
fixed-width family. "Coverage begins in the late twentieth century", above, is now a date rather
than a hedge.
