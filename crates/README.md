# crates

Rust crates implementing the domain computer — the retrieval, calculation, and feature
engineering capabilities that agents use to work with the corpus without loading it
wholesale into context.

See [crates conventions](../.yidam/.vendor/prelude/guidelines/directories.md) for the three
capability types (connectors, calculators, feature engineering) and the index layer.

This domain computer carries an unusual load: the corpus deliberately holds schema,
mechanism, and exemplars rather than bulk facts, so the per-agency-year numbers — roughly
610 education agencies across the years with usable data — live here as committed data
files and are queried through this layer. A corpus node cites a series; it does not restate
it.

**Connectors** adapt the primary publishers of Ohio funding data. All of them live in one crate,
[`connect`](connect/), as a registry whose status is checked by a test rather than asserted in a
README — and the table is now generated from that registry, because this paragraph is where the
count last went stale. Nine were approved at genesis and four since; most are wired, two are not,
and three more are wired for only part of what they feed and say so in a field rather than in
prose. The counts, the blockers and the gaps are in
[`connect/README.md`](connect/README.md); [`connect/sources/`](connect/sources/) says what each
one is for.

The two that are blocked are worth naming here, because they bound what the rest of this
workspace can compute. `dew-payment-reports` would carry the voucher and community-school
deduction per resident district, and sits behind the department's authenticated reports portal;
`ofcc-projects` is the whole capital channel, and its publisher serves interactive maps to a
browser and a 404 to anything that identifies itself.

**Calculators** are pure, deterministic, and the reason parameters are first-class nodes:

- `foundation` — re-runs a named funding regime for a given fiscal period against a
  parameter set; this is the simulation engine the `scenario` class binds to
- `local-capacity` — state share index and local wealth measures
- `millage` — effective millage under HB 920 reduction factors, including the 20-mill floor
- `dispersion` — equity statistics across agencies, operationalizing the `doctrine` nodes
- `deflator` — nominal-to-real normalization, without which a corpus spanning 1851 to the
  present cannot compare any two numbers honestly
- `project` — forward projection of enrollment with intervals, and the policy levers over it;
  it reports simulation and forecast separately and refuses to add them
- `scenario-delta` — the winners-and-losers table between two runs, with incidence across
  wealth and state share; a total cannot be constructed without the count of districts it
  fails to reach, and both orderings of the table are returned or neither is
- `regime-diff` — differences two regimes at component level, over the pairs the corpus's
  `replaces` graph actually aligns rather than a guessed correspondence; it carries the
  charge-off millage series with its statutory authority, and reports the residual its
  decomposition cannot explain

**`deduction` is declared and not built.** It was listed here as though it existed for long enough
that the web layer's "what is not modelled" note was written from this file rather than from the
workspace. There is a [skill describing what it would compute](../.yidam/skills/deduction.md) —
the amount and student count leaving a resident district through each `program`, and critically
the mechanism switch the Fair School Funding Plan made, after which community and STEM students
are funded directly rather than deducted. What is missing is the per-agency participation series
it would read. Until that exists, the 609 districts every calculator here covers are the
traditional districts in the department's own model, and the voucher channel is outside all of it.

A semantic index over `.yidam/corpus/` is not built: the corpus still fits in context. It is
added when direct retrieval stops working, which the exemplar-agency expansion will force before
anything else does. The node count lives in the generated block at the bottom of this file rather
than in this sentence, where it was wrong by twenty-nine.

## Workspace

The Rust workspace root is [`Cargo.toml`](Cargo.toml) in this directory, so all Rust lives
under `crates/` and no build configuration sits at the repository root. What does sit there is
[`mise.toml`](../mise.toml), which is a task runner rather than a build system: it addresses this
directory as the `//crates` subproject and shells out to the `cargo` that lives here.

Every directory here is now a real crate; the nine connector stubs were folded into
[`connect`](connect/) and their
prose kept at [`connect/sources/`](connect/sources/). The registry has since grown to **21**
connectors, and `sources/` still holds **11** long forms — for the other ten the decision record
is the only account of why each exists. The count is not repeated here on purpose: it is checked
by `registry::tests::every_connector_approved_in_the_ontology_is_present`, and the breakdown is
generated into [`connect/README.md`](connect/README.md).

**No external dependencies.** Every crate is pure `std` — including the XLSX reader, which
means a zip reader, a DEFLATE decompressor, and an XML parser written here rather than pulled
in. That keeps the domain computer hermetic and fast to build; it means a committed
[`scenario`](../.yidam/corpus/scenario/) result can be reproduced years later without a
dependency resolution succeeding first; and it means the *refresh* path keeps working too,
which matters more — an extraction pipeline that will not run is a corpus that cannot be
updated.

Two system binaries are used, named where they are used and needed by neither the build nor
the computation. **curl**, for HTTPS: TLS is the one thing in this pipeline that should not be
hand-written next to a DEFLATE decoder, and curl ships with macOS, Windows and every Linux
distribution, so only the *refresh* path needs it. **pdftotext**, for the PDF sources — and that
one is a weaker argument, because poppler ships with neither macOS nor Windows. Without it
`rebuild` reports the PDF fixtures skipped and continues, so
[`project/fixtures/appropriation-lines.csv`](project/fixtures/appropriation-lines.csv) is
regenerated **2,083 rows short** — the whole of the greenbook era, FY1999-FY2011 — while the
run still succeeds. See [`connect/src/cache.rs`](connect/src/cache.rs) for why that is the
chosen behaviour rather than a hard failure.

LibreOffice was a third until [`spreadsheet`](spreadsheet/) learned to read the pre-2007 `.xls`
format natively, so `rebuild` now regenerates every committed spreadsheet fixture from a checkout
with no external converter.

| Crate | Kind | Status |
|---|---|---|
| [`edfund-core`](edfund-core/) | types | shared `FiscalYear`, `AgencyType`, rounding |
| [`spreadsheet`](spreadsheet/) | reader | inflate, zip, XML, XLSX — no dependencies |
| [`connect`](connect/) | connectors | registry, cache, digests, fixture builders |
| [`deflator`](deflator/) | calculator | implemented; series verified against BLS |
| [`local-capacity`](local-capacity/) | calculator | the FSFP capacity measure; the charge-off it replaced is in [`regime-diff`](regime-diff/) |
| [`foundation`](foundation/) | calculator | full base cost build-up; verified to the cent |
| [`millage`](millage/) | calculator | implemented; verified on 606 real districts |
| [`dispersion`](dispersion/) | calculator | dispersion, correlation, partial, OLS |
| [`project`](project/) | calculator | projection, policy levers, the district crosswalk |
| [`scenario-delta`](scenario-delta/) | calculator | delta table, reach, incidence bands |
| [`regime-diff`](regime-diff/) | calculator | component alignment, charge-off rates, residual |
| [`bundle`](bundle/) | export | versioned feed and scenario checkpoints for [`web/`](../web/) |

Test counts are not in that table on purpose: they are derivable, they drifted by 41 across
four phases while nobody noticed, and they are now generated into the index at the bottom of
this file by `edfund-connect index`.

`spreadsheet` and `connect` are the retrieval side: everything that can fail, and nothing that
computes a funding figure. `bundle` is the export seam between the corpus and the web layer.
Those three are the crates with binaries; the calculators are libraries.

Run the gate from this directory:

```
mise run gate          # fmt-check, lint, test, doc — or `mise run //crates:gate` from the root
```

which is these four, and the fourth is not optional. The doc links are how a reader gets from a
calculator to the corpus node that says what it is for, and `cargo doc` has now gone red silently
twice:

```
cargo fmt -- --check
cargo clippy --all-targets -- -D warnings
cargo test
RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --workspace
```

### On floating point

Ohio's formulas are ratio-heavy, so this workspace uses `f64` with **explicit rounding at the
points the department rounds**, and proves correctness by reproducing published figures to the
cent rather than by relying on a fixed-point type. Where the department's worked examples show
a rounded intermediate, the code rounds there too — and there is a test asserting that doing it
the other way no longer matches.

The one place this leaks is decimal ties: `1.005` is stored just below the midpoint and rounds
down. That limitation is documented and tested in `edfund-core` rather than hidden, because a
future input landing on a genuine tie would need decimal arithmetic, not a different rounding
mode.

## Crates

<!-- REGEN: yidam crates-index
Regenerated by: `yidam crates-index`
Fields per crate: name, capability type (connector/calculator/feature-engineering/index),
                  description, key external dependencies, test coverage.
-->
| Crate | Description | `#[test]` fns |
|---|---|--:|
| [`bundle`](bundle/) | Export a versioned JSON feed of the corpus's district-level findings for the web layer | 50 |
| [`connect`](connect/) | Retrieval and extraction: the department's publications into committed fixtures | 109 |
| [`deflator`](deflator/) | Convert nominal Ohio school finance figures to constant dollars, fiscal-year aligned | 17 |
| [`dispersion`](dispersion/) | School finance equity statistics: dispersion and wealth neutrality across agencies | 151 |
| [`edfund-core`](edfund-core/) | Shared domain types for the Ohio education funding computer | 30 |
| [`foundation`](foundation/) | Fair School Funding Plan base cost build-up, per R.C. 3317.011 | 47 |
| [`local-capacity`](local-capacity/) | Fair School Funding Plan local capacity and state share, per R.C. 3317.017 | 23 |
| [`millage`](millage/) | Effective operating millage under H.B. 920 reduction factors, and 20-mill floor status | 19 |
| [`project`](project/) | Forward projection of funding inputs, and policy simulation over them | 277 |
| [`regime-diff`](regime-diff/) | Difference two funding regimes at component level, with the residual the decomposition does not explain | 36 |
| [`scenario-delta`](scenario-delta/) | Winners and losers between two funding runs, with incidence and the off-formula count | 25 |
| [`spreadsheet`](spreadsheet/) | Read the department's published workbooks with no dependencies | 79 |

12 crates, 863 test functions, no crates.io dependencies. `cargo test` reports a different total: it adds doc-tests and counts each integration binary separately.
<!-- /REGEN -->

## Index status

<!-- REGEN: yidam index-status
Regenerated by: `yidam index-status`
Fields: index backend, embedding model, indexed node count, freshness (HEAD vs last
        indexed commit), stale node count, retrieval latency (p50/p95 last benchmark).
-->
No semantic index is built. The corpus is 117 nodes and fits in context; an index is added when direct retrieval stops working, which has not happened.
<!-- /REGEN -->
