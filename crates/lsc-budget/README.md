# lsc-budget — connector (stub)

**Source.** Ohio Legislative Service Commission: budget analyses ("redbooks"), the Catalog of
Budget Line Items, fiscal notes, and the formula simulations published alongside each biennial
budget.

**Feeds.** [`legislation`](../../.yidam/corpus/legislation/),
[`fiscal-period`](../../.yidam/corpus/fiscal-period/),
[`program`](../../.yidam/corpus/program/), and the appropriation side of
[`parameter`](../../.yidam/corpus/parameter/fsfp-phase-in-percentage.yml).

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
  [`packages/`](../../packages/), not here; this connector consumes the extracted output.
- Offline mode required.

## Status

Stub. Approved in [decisions/proposals.yml](../../.yidam/decisions/proposals.yml); not
implemented.
