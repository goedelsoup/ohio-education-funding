# ohio-edfund-extract

Retrieval and extraction: fetch the department's publications, parse them, and write the CSV
fixtures the Rust crates compute against.

This is the Python half of the domain computer. It exists for ecosystem reasons — spreadsheet
formats — not because the logic belongs in a dynamic language. Everything that must be
deterministic and auditable lives in [`crates/`](../../crates/), and the CSV fixtures are the
boundary between the two. A CSV is legible in a diff; a binary would not be.

## Use

```
python3 -m ohio_edfund_extract --list       # the source registry
python3 -m ohio_edfund_extract --rebuild    # regenerate every committed fixture
python3 -m ohio_edfund_extract --rebuild --refresh   # re-download first
```

Sources are cached under `.cache/sources/` (gitignored), so a rebuild after the first fetch
makes no network requests — the offline mode the prelude requires of connectors.

Then re-run the Rust gate. The fixtures are pinned by tests, so a source revision that moves a
finding fails there rather than passing silently:

```
cd crates && cargo test
```

That pairing is the point. Extraction can be re-run freely because the findings are asserted
downstream; a number that changes has to be looked at.

## Why no dependencies

Standard library only. An XLSX is a zip of XML, so `zipfile` plus `xml.etree` reads it,
including cached formula results — which is what makes the department's own funding
calculator usable as a data source rather than only as a document.

The Rust workspace holds the same line for a different reason: a committed
[`scenario`](../../.yidam/corpus/scenario/) result should be reproducible years from now
without a dependency resolution succeeding first. Here it means the extraction runs anywhere
Python does, with no environment to build.

## Tests

```
python3 -m unittest discover -s tests -t .
```

Workbooks are constructed in-memory rather than committed as binary fixtures, so the tests
are hermetic and the expected structure is visible in the test file.

## What the tests are actually guarding

Three traps, each of which produces confident wrong numbers rather than an error:

- **`<10` is not zero.** Suppressed small counts parse to `None`. A caller that wants zero has
  to say so, because "fewer than ten, withheld" and "none" are different claims and summing
  them as zero understates any aggregate over small districts.
- **`Summary_SFPR` ships a `State of Ohio` aggregate row** with a numeric IRN, so it survives
  any digit filter. Counting it as a district double-counts every statewide total — which
  happened once during this corpus's development and inflated the guarantee by exactly 2×.
- **Missing is not zero in the output either.** A blank cell stays blank rather than becoming
  `0`, so a district with no reported valuation is distinguishable from one whose valuation is
  genuinely nil.

## One external requirement

The department still publishes enrollment data as pre-2007 `.xls`, which is OLE2 rather than a
zip and cannot be read here. `convert_legacy_xls` shells out to headless LibreOffice, which
must be installed separately. It converts to `.xlsx` rather than straight to CSV because the
CSV filter exports only the active sheet, and the enrollment workbook keeps district data on
the third of seven.

Grade-band ADM currently comes from the FY2027 calculator, which publishes it directly, so the
enrollment source is registered but not on the rebuild path. It is retained for the earlier
years the calculator does not cover.

## Adding a source

1. Add a [`.yidam/catalog/`](../../.yidam/catalog/) node describing what it is and what it can
   be trusted for.
2. Add a `Source` to `sources.py` pointing at it by slug. The catalog says what; this says
   where.
3. Write a builder in `fixtures.py` as a pure function from rows to rows, and test it.
4. Rebuild, run the Rust gate, and read what moved.
