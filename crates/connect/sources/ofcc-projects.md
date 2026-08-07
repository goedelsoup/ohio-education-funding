# ofcc-projects — connector

**Source.** Ohio Facilities Construction Commission, and its predecessor the Ohio School
Facilities Commission: project records, state share percentages, district wealth rankings, and
local share requirements.

**Feeds.** [`program`](../../../.yidam/corpus/program/classroom-facilities-assistance.yml),
[`education-agency`](../../../.yidam/corpus/education-agency/).

This is the only source for the capital channel, which is invisible in every operating
per-pupil figure and was itself part of the *DeRolph* remedy.

## Retrieval interface

```
fetch_projects(district?, fiscal_year?)  -> Vec<ProjectRecord>
fetch_wealth_ranking(fiscal_year)        -> Vec<RankingRecord>   // drives state share pct
fetch_state_share(district, fiscal_year) -> SharePercentage
```

`ProjectRecord` carries state share, local share, whether the local share required a bond
issue, and whether that issue passed. The last field is the one that connects the capital
channel back to [H.B. 920](../../../.yidam/corpus/legislation/hb-920-1976.yml): a program aimed
at property-poor districts requires those districts to pass a levy.

## Constraints

- Projects span multiple fiscal years. A record's fiscal year is the year of the funding
  action, not of completion, and totalling by year without care double-counts.
- Offline mode required.

## Status

**Declared.** Approved in [decisions/proposals.yml](../../../.yidam/decisions/proposals.yml);
no endpoint wired.

Project records sit behind a search form with no bulk export. The capital channel therefore
remains invisible in every per-pupil figure this repository computes, which is worth stating
plainly rather than leaving as an unremarked absence: capital was part of the *DeRolph* remedy,
and an operating-only view of Ohio school funding understates what the state did about it.
