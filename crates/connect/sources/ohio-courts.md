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

**Wired**, for the four DeRolph opinions — see
[`decisions/the-last-three-connectors`](../../../.yidam/decisions/the-last-three-connectors.yml).

The recorded blocker had two clauses of very different quality. *"Opinions are PDFs"* stopped being
a blocker the moment `Format::Pdf` had a reader; all four retrieve on a plain request. *"Trial-level
rulings such as the 2025 EdChoice decision are not in the supreme court archive at all"* is correct
and unfixable from here — a common pleas ruling is not in the supreme court's archive because it is
not the supreme court's. The `vouchers-hurt-ohio-2025` node stays sourced to reporting.

`citing_cases` is likewise unbuilt and needs a citator rather than a document.

What wiring bought: `regime_diff::RATES` carries the charge-off progression with a session-law
citation on each, sourced to DeRolph I ¶97, and until now that was a claim about a document nothing
here held. Every authority string is verbatim from the opinion, and the base — *"the charge-off is
the total taxable value of real and tangible personal property in the district times a certain
percentage"* — is the court's own words rather than the corpus's inference.
