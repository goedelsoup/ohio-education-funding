# ohio-courts — connector

**Source.** Supreme Court of Ohio opinion archive, plus courts of appeals and common pleas
dockets where a case has not reached the supreme court — the 2025 EdChoice decision is a
trial-level ruling and is not in the supreme court archive.

**Feeds.** [`litigation`](../../../.yidam/corpus/litigation/).

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

**Wired**, for the four DeRolph opinions and the EdChoice challenge's one appellate decision — see
[`decisions/the-last-three-connectors`](../../../.yidam/decisions/the-last-three-connectors.yml) and
[`decisions/what-a-citator-reaches`](../../../.yidam/decisions/what-a-citator-reaches.yml).

The recorded blocker had three clauses and none survived contact in the form it was written.
*"Opinions are PDFs"* stopped being one the moment `Format::Pdf` had a reader. The other two were
assumptions and are now measurements.

### The trial-level ruling is a licence problem, not an archive problem

*"Trial-level rulings such as the 2025 EdChoice decision are not in the supreme court archive at
all"* is true and is not the reason the ruling is absent from this repository. The Reporter of
Decisions' own source list is the Supreme Court, the twelve district courts of appeals, the Court
of Claims and Miscellaneous — **no common pleas court**, so no trial ruling of any kind is there.

What publishes the ruling is the **Franklin County Clerk of Courts' Case Information Online**,
which returns 200 to this project's user agent and whose conditions of use say:

> Data and information from CIO is not intended for distribution by other persons, entities or
> organizations … Any public or private organization or individual(s) wishing to obtain data files
> from the Franklin County Clerk of Court's Office must submit a Public Records Request.

So a person may read it and this repository may not redistribute it — the same shape as the
educational service center minute books, and remediable the same way, by a request under
R.C. 149.43.

**What is wired instead is the case's appellate record.** *Columbus City School Dist. v. State*,
2024-Ohio-1217, 10th Dist. No. 24AP-60, is a decision in this very case, published by the Reporter
at the same URL shape as the four DeRolph opinions. It is about a deposition subpoena served on the
President of the Ohio Senate and it dismisses for want of a final appealable order — so it settles
nothing about EdChoice and settles two things `vouchers-hurt-ohio-2025` carried as `[open]`: the
caption, and the trial court case number, **Franklin C.P. No. 22CV-000067**.

### `citing_cases` needs a citator, and one exists

[CourtListener](https://www.courtlistener.com), run by the Free Law Project, is a free citator that
covers Ohio and serves this project's user agent without complaint. It holds 87,949 Ohio Supreme
Court opinions and 110,756 Ohio Court of Appeals opinions, and its `/api/rest/v4/search/` endpoint
answers anonymously.

**The property stays unfilled anyway, and now for a measured reason.** Against DeRolph I:

| | Ohio decisions other than DeRolph's own |
|---|---|
| in CourtListener's citation graph | **10** |
| containing `78 Ohio St.3d 193` in its own full text | **25** |

The graph is a strict subset of the text hits — nothing in it is missing from them. The fifteen it
does not have include ***Simmons-Harris v. Goff*** (1999), the Ohio Supreme Court's own voucher
decision, and ***State ex rel. Ohio Congress of Parents & Teachers v. State Bd. of Edn.*** (2006),
the community-school constitutional challenge. Those are the two Ohio cases a reader of this corpus
would most want on that edge. A `citing_cases` list built from this citator would be silently
missing them.

### Six things about the citator worth not working out twice

- **`cites:` takes opinion ids, not cluster ids.** A DeRolph cluster holds five opinions. Querying
  the lead opinion alone returns **0**; querying all five siblings returns 29. The wrong query
  returns a plausible zero rather than an error.
- **Each DeRolph opinion is in the database twice** — once from the reporter feed and once from
  Ohio's WebCite feed — and the citation counts split across the two clusters (129 and 3 for
  DeRolph I).
- **`citeCount` and the search count measure different things.** `citeCount` counts citing
  *opinions*, including each concurrence separately; the search counts *decisions*.
- **A nonexistent court id returns `count: 0` under HTTP 200.** A coverage negative from this API
  is worth nothing without a positive control, which is how the Ohio common pleas question was
  nearly answered wrongly here.
- **Anonymous requests are rate-limited to five a minute**, which is a real constraint on a
  connector that would walk a citation graph.
- **`/opinions-cited/` and `/clusters/` return 401**; they want a free API token this repository
  does not hold. And `courtlistener.com` itself — the human site, as distinct from the API — is
  **403 CDN-blocked** for a non-browser agent, so only the API is reachable.

What wiring bought: `regime_diff::RATES` carries the charge-off progression with a session-law
citation on each, sourced to DeRolph I ¶97, and until now that was a claim about a document nothing
here held. Every authority string is verbatim from the opinion, and the base — *"the charge-off is
the total taxable value of real and tangible personal property in the district times a certain
percentage"* — is the court's own words rather than the corpus's inference.
