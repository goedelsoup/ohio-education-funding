# LSC appropriation spreadsheet

**Publisher.** Ohio Legislative Service Commission, Legislative Budget Office.

**What it is.** The line-item detail behind each main operating budget: every appropriation line
item in state government, by fund group and fund, with an amount per fiscal year. Published
alongside the budget act and indexed at `/budget/{ga}/main-operating-budget`. Called
*budget in detail* through the 132nd General Assembly and *appropriation spreadsheet* from the
133rd; the two names are the same artifact.

**Coverage held here.** The 129th General Assembly through the 136th — FY2012-13 through
FY2026-27 — filtered to the Department of Education's line items, which are the ones numbered
`200xxx`. Earlier bienniums back to the 124th exist as greenbook PDFs and are not extracted; see
[`the-greenbook-series`](../decisions/the-greenbook-series.yml).

## What it can be trusted for

The enacted appropriation, the actual expenditure, and the adjusted appropriation for a named line
item in a named fiscal year — **provided the right variant is read**, which is the whole difficulty
with this source.

LSC publishes each biennium twice:

| Variant | What it states | What it does not |
|---|---|---|
| `as enacted` / `as enrolled` | The bill's whole path: introduced, each chamber's substitute and report, conference, and as enacted | Nothing about what was later spent |
| `with actual expenditures and adjusted appropriations` | Actual spending for closed years, and the adjusted appropriation for the year still open | **The enacted figure, for any year that has closed** — the column is overwritten |

The second point is the trap. The revised workbook looks like a superset and is not: for the
132nd, it reports FY2018 as an actual and FY2019 as an adjusted appropriation, and states what was
enacted for that biennium nowhere at all. Both variants are held here for that reason.

## Caveats

- **A fiscal year does not identify a column.** These workbooks carry `$ Change` and `% Change`
  columns headed with a year, and the `as enacted` variant carries an amount for every stage the
  bill passed through. Only the enacted, actual and adjusted columns are extracted; the
  legislative stages are real figures that never became law and are left in the source.
- **An unlabelled column means different things in the two variants.** In the revised workbook a
  bare `FY 2014` is spending; in the enacted one it is a prior year. Nothing in the cell says so.
- **The 129th serves the same file under both names.** Its `as-enrolled` and
  `with-actual-expenditures` URLs return byte-identical content, so FY2012-13 has actuals and no
  enacted figure anywhere in this series.
- **Delivered as OLE2 under an `.xlsx` name** for the 129th. The format is settled by the file's
  leading bytes rather than its extension.
- These are appropriations, not payments. What a district received is `dew-foundation`; what the
  General Assembly appropriated to a line is this. They answer different questions and a
  difference between them is not an error in either.

## Verification

Sixteen documents across eight bienniums overlap heavily: a fiscal year is reported by up to four
of them, as an enacted amount in the act that made it and as a prior-year actual in later ones.
943 claims are corroborated by at least a second document and **none of them disagree** —
`crates/project/tests/the_appropriation_series.rs` asserts it on every run.

## Retrieval

`edfund-connect fetch <bill>-enacted` and `<bill>-actuals`, for each of `hb153`, `hb59`, `hb64`,
`hb49`, `hb166`, `hb110`, `hb33`, `hb96`. Digest-pinned in
[`source-digests.txt`](../../crates/connect/source-digests.txt).
