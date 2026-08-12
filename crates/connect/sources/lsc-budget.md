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

**Wired**, for the **current biennium**: the H.B. 96 final analysis, which is where every
provision the Revised Code does not contain actually lives, and the H.B. 96 redbook, which is the
bill **as introduced**.

The recorded blocker said the rest are PDFs. That is true, and was treated as the end of the
matter; a PDF is a container, and `Format::Pdf` now has a reader. It also said the redbooks were
unretrieved while one sat in the registry, and that the pre-2000 record was out of reach. All
three claims were replaced after somebody looked — see
[`the-greenbook-series`](../../../.yidam/decisions/the-greenbook-series.yml).

### What is actually there

LSC publishes an education budget analysis for **every main operating budget from the 124th
General Assembly to the 136th** — thirteen consecutive bienniums, FY2002-03 through FY2026-27 —
indexed at `/budget/{ga}/main-operating-budget`, with
`/budget/appropriation-biennium-reference-list` mapping each General Assembly to its biennium.
All thirteen retrieve on a plain request.

They are **greenbooks**, published *as enrolled*. That is the distinction that matters: the
redbook wired here is *as introduced*, so its amounts are the executive proposal, and every corpus
claim resting on it says so. A greenbook for H.B. 96 sits at a sibling URL to the redbook already
in the registry.

**Two variants per biennium, and only one of them carries the enacted figure.** This was
established by wiring all eight workbooks and reading the result: the
`with-actual-expenditures-and-adjusted-appropriations` variant is revised after the biennium
closes and *replaces* the enacted column with actuals, so the 132nd's workbook states the enacted
FY2018-19 appropriation nowhere. Both variants are needed — sixteen workbooks, not eight — and a
fiscal year does not identify a column: three of them carry more than one column naming the same
year, and the extractor has to refuse on that rather than emit all of them.

From the **129th onward the line-item detail is also a workbook** — `budget-in-detail` through the
132nd, `appropriation-spreadsheet` from the 133rd — including a variant with **actual expenditures
and adjusted appropriations** beside the enacted amounts, in columns
`Budget | Fund Group | Fund | ALI | ALI Name | FY ...`. [`spreadsheet`](../../spreadsheet/) reads
that natively, so two thirds of the series needs no text extraction at all.

The remaining five bienniums (124th-128th, FY2002-FY2011) are greenbook PDFs. Every one parses —
63 to 153 distinct appropriation line items each — **by column position, not by token order**.
`pdftotext -layout` leaves an unfunded year as an empty column and prints `N/A` in `% Change`, so
counting dollar tokens left to right silently shifts a row's fiscal years. A misdated appropriation
is not a parse failure and nothing downstream would catch it.

### What is still blocked

Anything **before FY1999**. The greenbook series begins at the 124th and its earliest column is
FY1999 actuals, so the Foundation Program era, *DeRolph I* and the equal yield formula still need
the session laws.

The **Catalog of Budget Line Items** and the **per-district simulations** each have an index page —
`/budget/catalog-of-budget-line-items` and the per-bill document lists — and neither has been
opened.
