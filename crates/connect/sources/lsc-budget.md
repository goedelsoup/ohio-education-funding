# lsc-budget — connector

**Source.** Ohio Legislative Service Commission: budget analyses ("redbooks"), the Catalog of
Budget Line Items, fiscal notes, and the formula simulations published alongside each biennial
budget.

**Feeds.** [`legislation`](../../../.yidam/corpus/legislation/),
[`fiscal-period`](../../../.yidam/corpus/fiscal-period/),
[`program`](../../../.yidam/corpus/program/), and the appropriation side of
[`parameter`](../../../.yidam/corpus/parameter/fsfp-phase-in-percentage.yml).

LSC is the only source with a continuous appropriation-line series across the whole period, and
its simulations are frequently the only published estimate of what a formula would produce
before a fiscal year closes. It is also the primary source for the pre-2000 record.

## Retrieval interface

```
fetch_line_items(biennium, agency?)      -> Vec<LineItemRecord>
fetch_simulation(bill, fiscal_year)      -> Vec<SimulationRecord>   // per-district estimates
fetch_analysis(bill)                     -> AnalysisDocument
```

`SimulationRecord` must be tagged as an estimate, never merged into a realized series from
`dew-foundation`. An LSC pre-enactment simulation and a department payment report describe the
same quantity and routinely disagree; the corpus keeps both and names which is which.

## Constraints

- Much of the pre-2000 material is scanned PDF. Table extraction belongs in
  [`packages/`](../../spreadsheet/), not here; this connector consumes the extracted output.
- Offline mode required.

## Status

**Wired**, for **one document**: the final analysis of the current budget act, which is where
every provision the Revised Code does not contain actually lives.

The recorded blocker said these are PDFs. That is true, and was treated as the end of the matter;
a PDF is a container, and `Format::Pdf` now has a reader.

**What is still ahead is the reason this connector exists.** The redbooks, the Catalog of Budget
Line Items and the per-district simulations remain unretrieved, so the continuous
appropriation-line series is not built and the pre-2000 record is not here. That is recorded in
the registry as `still_blocked` rather than left to this paragraph, so `edfund-connect list` does
not read as though the appropriation series were served.

The obstacles differ by document. Redbooks and the Catalog are PDFs, now tractable in principle.
The per-district simulations are workbooks posted per bill with no index — the valuable part and
the most tractable in format, since they are spreadsheets and
[`spreadsheet`](../../spreadsheet/) already reads that format. What is missing there is a way to
enumerate them. Much of the pre-2000 material is scanned PDF, which is a different problem again.
