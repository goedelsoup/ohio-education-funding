# ohio-courts — connector (stub)

**Source.** Supreme Court of Ohio opinion archive, plus courts of appeals and common pleas
dockets where a case has not reached the supreme court — the 2025 EdChoice decision is a
trial-level ruling and is not in the supreme court archive.

**Feeds.** [`litigation`](../../.yidam/corpus/litigation/).

## Retrieval interface

```
fetch_opinion(citation)            -> OpinionDocument
fetch_docket(court, case_number)   -> DocketRecord      // for pending and trial-level matters
citing_cases(citation)             -> Vec<Citation>     // populates the `cites` edge
```

`citing_cases` is what makes the precedent chain traversable rather than hand-maintained.

## Constraints

- Coverage is uneven by court level. Supreme court opinions are well structured; common pleas
  decisions often are not, and may require manual entry with a catalog anchor.
- Appellate status changes. A cached decision must carry a freshness marker — the 2025 EdChoice
  ruling is under appeal, and a stale cache would present it as settled.
- Offline mode required.

## Status

Stub. Approved in [decisions/proposals.yml](../../.yidam/decisions/proposals.yml); not
implemented.
