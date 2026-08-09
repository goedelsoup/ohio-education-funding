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

The blocker recorded here for twelve phases — *"project records sit behind a search form with no
bulk export"* — was a guess, and it was wrong about the first obstacle.

**The blocker is now precise, and it is partly a choice.** `ofcc.ohio.gov` returns 404 to this
project's contactable user-agent and 200 to a browser string. The data is served; the agent is
filtered. Sending a browser string would work, and [`cache`](../src/cache.rs) already states the
position on that — impersonating one *"would be discourteous besides"*. The filter is more likely
an undiscriminating CDN default than a considered exclusion of researchers, but guessing at intent
is not a reason to route around it.

Behind the filter the project portfolios are interactive maps rather than files, so a bulk export
would still have to be reverse-engineered out of a map service. The honest next step is to ask the
commission, not to change the agent string.

The capital channel therefore remains invisible in every per-pupil figure this repository
computes, which is worth stating plainly rather than leaving as an unremarked absence: capital was
part of the *DeRolph* remedy, and an operating-only view of Ohio school funding understates what
the state did about it.
