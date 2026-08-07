# census-f33 — connector

**Source.** U.S. Census Bureau, Annual Survey of School System Finances (the F-33 series):
revenue by source and expenditure by function, for every school system in the country, on a
consistent definition.

**Feeds.** [`metric`](../../../.yidam/corpus/metric/),
[`education-agency`](../../../.yidam/corpus/education-agency/).

Its value is comparability in two directions. Across states, it is the only way to say whether
Ohio's arrangement is unusual. Within Ohio, it is an independent check on department figures
computed on different definitions — where the two disagree, the disagreement is informative.

## Retrieval interface

```
fetch_finance(fiscal_year, state?, leaid?) -> Vec<F33Record>
crosswalk(leaid)                           -> Option<Irn>
```

## Constraints

- Roughly a two-year publication lag. Never the source for a current-year figure.
- Census definitions differ from Ohio's. F-33 "current spending" is not the operating cost the
  Ohio formula computes, and the two must not be compared without stating the definitional gap.
- Keys on federal LEAID, not IRN. The crosswalk is required and is not always one-to-one.
- Offline mode required.

## Status

**Retrievable, not parsed.** A URL for the FY2022 table is in the registry and
`edfund-connect fetch f33-fy2022` works. Nothing reads it.

What blocks a parser is not the format but the *series*: the layout changes across years, so
useful coverage means a per-era parser rather than one, and the connector's whole value is
comparability across a long span. One year read badly is worth less than no years.
