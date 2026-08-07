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

**Connectors** adapt the primary publishers of Ohio funding data. All nine approved at genesis
now live in one crate, [`connect`](connect/), as a registry whose status is checked by a test
rather than asserted in a README. Two are wired — `dew-foundation` and `bls-cpi` — one is
retrievable, and six are declared with a recorded reason. See
[`connect/README.md`](connect/README.md) for the table and
[`connect/sources/`](connect/sources/) for what each is for.

**Calculators** are pure, deterministic, and the reason parameters are first-class nodes:

- `foundation` — re-runs a named funding regime for a given fiscal period against a
  parameter set; this is the simulation engine the `scenario` class binds to
- `local-capacity` — state share index and local wealth measures
- `millage` — effective millage under HB 920 reduction factors, including the 20-mill floor
- `deduction` — community school and scholarship deductions against a resident district
- `dispersion` — equity statistics across agencies, operationalizing the `doctrine` nodes
- `deflate` — nominal-to-real normalization, without which a corpus spanning 1851 to the
  present cannot compare any two numbers honestly
- `project` — forward projection of enrollment with intervals, and the policy levers over it;
  it reports simulation and forecast separately and refuses to add them

A semantic index over `.yidam/corpus/` is not built at genesis — 45 nodes fit in context.
It is added when the corpus outgrows direct retrieval, which the exemplar-agency expansion
will force before anything else does.

## Workspace

The Rust workspace root is [`Cargo.toml`](Cargo.toml) in this directory, so all Rust lives
under `crates/` and the repository root stays free of build configuration. Every directory here
is now a real crate; the nine connector stubs were folded into [`connect`](connect/) and their
prose kept at [`connect/sources/`](connect/sources/).

**No external dependencies.** Every crate is pure `std` — including the XLSX reader, which
means a zip reader, a DEFLATE decompressor, and an XML parser written here rather than pulled
in. That keeps the domain computer hermetic and fast to build; it means a committed
[`scenario`](../.yidam/corpus/scenario/) result can be reproduced years later without a
dependency resolution succeeding first; and it means the *refresh* path keeps working too,
which matters more — an extraction pipeline that will not run is a corpus that cannot be
updated.

Two system binaries are used, both named where they are used and neither needed to build or
compute: **curl** for HTTPS, and **LibreOffice** for the one source still published in the
pre-2007 `.xls` format.

| Crate | Kind | Status | Tests |
|---|---|---|---|
| [`edfund-core`](edfund-core/) | types | shared `FiscalYear`, `AgencyType`, rounding | 7 |
| [`spreadsheet`](spreadsheet/) | reader | inflate, zip, XML, XLSX — no dependencies | 47 |
| [`connect`](connect/) | connectors | registry, cache, digests, fixture builders | 58 |
| [`deflate`](deflate/) | calculator | implemented; series verified against BLS | 11 |
| [`local-capacity`](local-capacity/) | calculator | FSFP side implemented; charge-off side not | 16 |
| [`foundation`](foundation/) | calculator | full base cost build-up; verified to the cent | 43 |
| [`millage`](millage/) | calculator | implemented; verified on 606 real districts | 13 |
| [`dispersion`](dispersion/) | calculator | implemented; verified on 606 real districts | 20 |
| [`project`](project/) | calculator | forward projection and policy simulation | 39 |
| [`bundle`](bundle/) | export | versioned JSON feed for [`web/`](../web/) | 6 |

`spreadsheet` and `connect` are the retrieval side: everything that can fail, and nothing that
computes a funding figure. `bundle` is the export seam between the corpus and the web layer.
Those three are the crates with binaries; the calculators are libraries.

Run the gate from this directory:

```
cargo fmt -- --check
cargo clippy --all-targets -- -D warnings
cargo test
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
_Run `yidam crates-index` to populate._
<!-- /REGEN -->

## Index status

<!-- REGEN: yidam index-status
Regenerated by: `yidam index-status`
Fields: index backend, embedding model, indexed node count, freshness (HEAD vs last
        indexed commit), stale node count, retrieval latency (p50/p95 last benchmark).
-->
_Run `yidam index-status` to populate._
<!-- /REGEN -->
