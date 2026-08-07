# spreadsheet — reading the department's workbooks

Ohio publishes its funding data as Excel workbooks. Reading one needs a zip reader, a DEFLATE
decompressor, and an XML parser. This crate is all three, with no dependencies.

| Module | What it is |
|---|---|
| [`inflate`](src/inflate.rs) | DEFLATE decompression, RFC 1951 |
| [`zip`](src/zip.rs) | zip central directory, stored and deflated members, CRC-32 |
| [`xml`](src/xml.rs) | a pull parser for the subset SpreadsheetML uses |
| [`xlsx`](src/xlsx.rs) | shared strings, sheet list, rows |

## Why write this rather than depend on it

The workspace's rule is that a committed [`scenario`](../../.yidam/corpus/scenario/) result
should be reproducible years from now without a dependency resolution succeeding first. That
rule matters at least as much on the retrieval side: an extraction pipeline that will not run
is a corpus that cannot be refreshed.

It is also less code than it sounds. DEFLATE is a small format — three block types, two Huffman
tables, one back-reference rule — and the XML that SpreadsheetML emits is a small subset with no
namespace resolution needed. The whole crate is about a thousand lines and reads a 5 MB,
twenty-six-sheet workbook in under a second.

## Cached formula results are the point

A cell holding a formula stores both the formula in `<f>` and the value the last recalculation
produced in `<v>`. This reader takes `<v>`. That is what makes the department's *own funding
calculator* — a workbook that is almost entirely formulas — a data source rather than a
document, with no spreadsheet engine anywhere in the pipeline.

The trade is that the values are whatever the department last recalculated and saved. They are
the published figures, which is what the corpus wants, but a workbook saved with stale
calculation would read stale.

## The CRC is checked

Every zip member carries a CRC-32 of its uncompressed bytes, and this verifies it. That costs
one pass over data already in memory and turns a subtle decompression bug into a loud failure —
worth paying for when the decompressor is hand-written for this workspace, because the
alternative is a wrong number in a fixture that no test would think to question.

## What it does not do

No compression, no encryption, no Zip64 beyond reading the extra field, no OLE2. Legacy `.xls`
is a different format entirely and lives in [`connect::legacy`](../connect/src/legacy.rs),
where the gap is documented rather than papered over.

This layer knows about *formats*. It knows nothing about Ohio — no IRNs, no fiscal years, no
suppressed-count conventions. Those live in [`connect`](../connect/), where they can be revised
when a publication changes shape.

## Tests

Hermetic. DEFLATE is checked against committed byte vectors produced once by zlib; zip and XLSX
tests build stored-method archives in memory, which needs no compressor and keeps the expected
structure visible in the test rather than in a binary fixture.
