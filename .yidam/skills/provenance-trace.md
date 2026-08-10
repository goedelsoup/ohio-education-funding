---
name: provenance-trace
description: Follow a numeric claim from a corpus node back to its catalog entry, the published file, and the digest that pins the bytes it was read from
---

# Procedure: provenance-trace

**Purpose.** Establish, for one figure, exactly what it was read from — and whether that thing
still says it.

**Composes.** the [`catalog`](../catalog/), `crates/connect/source-digests.txt`, and the `Source`
records in the connector registry.

## Steps

1. **Node to catalog, in one hop.** The corpus rule is that every numeric claim reaches a
   [catalog entry](../catalog/) directly. If it takes two, the node is under-cited.
2. **Catalog to source.** The catalog entry says what the publication is and what it can be
   trusted for. A `Source` in the registry names the URL, the format, and every fixture built
   from it.
3. **Source to bytes.** `source-digests.txt` records the SHA-256 and byte length of the exact
   published file the fixture was built from. Run `edfund-connect verify`.
4. **Read the mismatch, do not clear it.** A digest mismatch means *the publication was revised* —
   the department reposts corrected workbooks at the same URL. That is a diff to read, not a
   manifest to overwrite. `verify --write` after reading, never before.

## Why one hop, and why this domain in particular

A foundation payment report, an LSC estimate made before the fiscal year closed, and a district's
own five-year forecast are three numbers describing the same thing, and they routinely disagree.
A figure without provenance is not merely unsourced here — it is ambiguous between three defensible
values.

Provenance attaches to the **artifact, not the connector**, because one connector retrieves
publications with different caveats: `dew-foundation` fetches a projection (the FY2027 model), a
profile of actuals (the Cupp report), and a headcount, and no single trust statement covers all
three.

## What a trace can conclude

- **Traced** — node → catalog → source → digest, and `verify` agrees.
- **Revised** — the chain holds but the publisher's file has changed. Report the diff; this is
  informative, not a failure.
- **Unanchored** — the fixture is committed and no `Source` declares it. This is the condition the
  provenance test now makes impossible in both directions, and it existed:
  `expenditure-functions-fy25.csv` was rebuilt on every run and declared by nothing, because a
  `Source` could name only one fixture while three sources feed several.

## Refusals

- **Do not treat a corpus node as provenance for another corpus node.** The graph is not a source.
- **Do not accept a citation without checking the cited thing exists.** Two `statutory_basis`
  fields pointed at Revised Code sections that do not exist, both plausible-looking numbers in the
  right chapter. A citation is only checkable against the thing it cites.
