# connect — retrieval and extraction

The side-effecting half of the domain computer. It fetches what Ohio and the federal government
publish, reads it with [`spreadsheet`](../spreadsheet/), and writes the CSV fixtures the
calculator crates compute against.

```
edfund-connect list                 what is retrievable, and how far each connector got
edfund-connect fetch <source>       download into .cache/sources (only this touches the network)
edfund-connect rebuild              regenerate the committed fixtures from cached sources
edfund-connect verify [--write]     check cached sources against the committed digests
edfund-connect cpi                  check the deflator series against the Bureau's file
edfund-connect head <source> <sheet> [n]
                                    dump a sheet with column indices, for mapping a new layout
```

Run it from `crates/`: `cargo run -p connect --bin edfund-connect -- list`.

## What replaced the stubs

Nine connectors were approved at genesis. For eleven phases all nine were directories holding a
README that described a retrieval interface nobody had written. That is now a
[registry](src/registry.rs) that runs, and each connector carries a **status that a test
checks** rather than a status that a reader has to take on faith:

| Connector | Status | What it retrieves |
|---|---|---|
| [`dew-foundation`](sources/dew-foundation.md) | **wired** | FY2027 funding calculator, District Profile Report, October headcount |
| [`bls-cpi`](sources/bls-cpi.md) | **wired** | CPI-U all items, from the Bureau's flat file |
| [`census-f33`](sources/census-f33.md) | retrievable | one year of the school finance survey; no parser |
| [`tax-abstract`](sources/tax-abstract.md) | declared | — |
| [`lsc-budget`](sources/lsc-budget.md) | declared | — |
| [`ohio-laws`](sources/ohio-laws.md) | declared | — |
| [`ohio-courts`](sources/ohio-courts.md) | declared | — |
| [`ofcc-projects`](sources/ofcc-projects.md) | declared | — |
| [`nces-ccd`](sources/nces-ccd.md) | declared | — |

A `declared` connector says **what blocks it** — that string is a field on the record, and a
test fails if it is missing or too short to mean anything. Seven of nine being blocked is the
honest state; the reasons are mostly "published as PDF" and "no bulk export", which is what
public data in this domain actually looks like.

Every source a committed fixture is built from must name a
[`.yidam/catalog/`](../../.yidam/catalog/) node, and a test asserts the node exists. Provenance
attaches to the artifact rather than the connector, because `dew-foundation` retrieves three
publications with three different sets of caveats.

## The digest manifest

[`source-digests.txt`](source-digests.txt) records the SHA-256 and byte length of the exact
published file each fixture was built from. The department reposts corrected workbooks at the
same URL, so without it "regenerated from the FY2027 calculator" names a moving target.
`verify` recomputes and compares; a mismatch is reported as *the publication was revised*,
which is a thing to read a diff about, not a thing to overwrite.

## Refresh and verification check each other

Extraction can be re-run freely **because** the findings are pinned downstream. A source
revision that moves the guarantee count, the millage floor count, or a scenario result fails a
test in [`foundation`](../foundation/) or [`dispersion`](../dispersion/) rather than passing
silently into the corpus.

That is not hypothetical. Rebuilding these fixtures in Rust changed nine lines of
`fy27-department-model.csv`, all in one column: the predecessor formatter trimmed trailing
zeros off integers, so a district with **10** school buildings was recorded as **1** and one
with 30 as 3. Nine districts carried a wrong building count. Nothing noticed, because building
count feeds the leadership sub-components and the verified figure was teacher base cost.

## Three conventions that cost something to learn

They are in [`conventions.rs`](src/conventions.rs) as tests, not comments, because each one
produces a confidently wrong number rather than an error:

- **`<10` is a suppressed count, not zero.** It parses to `None`. Summing it as zero
  understates any aggregate over small districts — which are exactly the districts a
  school-funding question is usually about.
- **`Summary_SFPR` ships a `State of Ohio` row with a numeric IRN**, so it survives any digit
  filter. Counting it as a district put the guarantee at exactly twice its real size once.
- **Missing stays blank in the output**, so a district with no reported valuation stays
  distinguishable from one whose valuation is nil.

## Dependencies

None from crates.io. Two on the system, both named where they are used:

- **curl**, for HTTPS. TLS is the one thing in this pipeline that should not be hand-written
  next to a DEFLATE decoder. See the module note in [`cache.rs`](src/cache.rs).
- **LibreOffice**, for one source. The department still publishes October enrollment as a
  pre-2007 OLE2 file, which is a different format entirely. Reading it natively means an OLE2
  sector walker and a BIFF8 record parser; that is the honest completion of `dew-foundation`
  and it is not done. It is why `rebuild` regenerates two of the three department fixtures.

Set `EDFUND_CONTACT` to an email address before fetching from the Bureau of Labor Statistics:
it rejects any request whose `User-Agent` has no contact, with a bare 403 and no explanation.

## What is not here

`fetch_deductions` — the community school and scholarship deduction series behind the voucher
channel. It is the largest remaining hole in `dew-foundation`, and the reason the `deduction`
calculator is still a stub.
