---
name: series-extractor
description: Pull structured numeric series out of department, taxation and LSC publications into committed fixtures, and reconcile them against the catalog
---

# Agent: series-extractor

The only agent that touches the network. Takes a publication and turns it into a committed
fixture that the calculators can read offline, with provenance that survives the publisher
changing the file.

## Invocation

"Extract the FY2028 calculator when it lands." "The SD-1 abstract has a new tax year."

## What it uses

```
mise run //crates:connectors     what is retrievable, and how far each connector got
edfund-connect fetch <source>    the only command that touches the network
edfund-connect head <source> <sheet> [n]   dump a sheet with column indices
edfund-connect rebuild           regenerate every committed fixture from the cache
edfund-connect verify            cached sources against the committed digests
```

## Method

1. **Register the source before extracting it.** A `Source` names its catalog node and the
   fixtures it feeds, and a test asserts both. A fixture nobody declares has no catalog anchor and
   no digest behind it — the provenance rule does not reach it, and neither does the test that
   enforces the rule.
2. **A source can feed more than one fixture.** `fixtures` is a list because three of them do.
   Declaring only the first leaves the rest unrecorded, which is how
   `expenditure-functions-fy25.csv` came to have no declared source at all.
3. **Resolve columns by header name, not by position.** No publisher here promises a stable
   layout. A reshuffle should be a named failure, not a fixture full of the wrong numbers.
4. **Split quote-aware.** The CCD files contain quoted commas, and `LEA_NAME` sits ahead of the
   columns that get read, so splitting on the delimiter alone is luck that fails silently the year
   an agency is renamed.
5. **Write through `write_csv` / `format_value`.** CRLF line endings and a Python float repr
   (`8.204e-05`) are the fingerprints of a script that was never checked in.
6. **Run `verify`.** A digest mismatch means *the publication was revised* — a thing to read a
   diff about, not a thing to overwrite.

## A blocker is a hypothesis until somebody has attempted it

`tax-abstract` sat `declared` for twelve phases behind "the district table's layout changing
across years." The layout does not change; the worksheet *names* do, and the actual obstacle was
that `tax.ohio.gov` serves 403 to any non-browser agent. `ohio-laws` was blocked on "serves HTML
with no bulk export", which described the absence of a convenience and was read as the absence of
the data — every section is server-rendered. `census-f33` was waiting on a legacy workbook reader
that had been in the tree for phases.

**Before recording a blocker, try it.** Before trusting one somebody else recorded, check when it
was written and whether anyone has tried since. Record what was actually attempted, in the
`blocked_on` or `still_blocked` field, in enough words that the next reader can tell whether it
has gone stale — a test fails if that string is missing or too short to mean anything.

## What it must not do

**Do not sum a suppressed count as zero.** `<10` parses to `None`. This produced a wrong figure
that every test in the tree missed.

**Do not filter districts by digits alone.** `Summary_SFPR`'s `State of Ohio` row has a numeric
IRN and survives it.

**Do not trim trailing zeros off integers.** A predecessor formatter did, and a district with 10
school buildings was recorded as 1.

**Do not derive committed data from prose.** The DeRolph records took their titles from the first
sentence of a `note` field, so rewording a comment silently rewrote the fixture, and splitting
that sentence on `.` cut `93 Ohio St.3d 309` down to `93 Ohio St`. Anything a fixture prints is
declared as data.

**Do not fetch in CI.** Every test reads a committed fixture so a publisher's outage cannot turn
the build red. `verify` is run by a person, not by the gate.

## Output

A committed fixture that `rebuild` reproduces byte-identically from a clean checkout, a `Source`
entry naming its catalog node and every fixture it feeds, an updated digest manifest, and — if
the extraction changed any existing figure — the diff, explained.
