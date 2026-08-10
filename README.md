# Ohio education funding

How Ohio funds its public schools, traced from the statute to the district.

The state foundation formula and its successive regimes; the local property tax base that
[H.B. 920](.yidam/corpus/legislation/hb-920-1976.yml) froze against inflation; the litigation that
found the combination unconstitutional and then stopped without a compliance finding; and the
programs that route money around the formula's edges.

It is built so that a specific dollar figure can be walked back to the parameter and the section
that produced it, and so that "what would this change do to my district?" is a question with an
answer rather than an opinion. The site is at
**<https://schools.ohio.shawneesmart.systems>**.

## What is here

| Directory | What it holds |
|---|---|
| [`.yidam/corpus/`](.yidam/corpus/) | The knowledge graph: 76 nodes across 13 classes — regimes, components, parameters, litigation, doctrine, exemplar districts |
| [`.yidam/catalog/`](.yidam/catalog/) | 27 source records. Every numeric claim in the corpus should reach one in a single hop |
| [`.yidam/decisions/`](.yidam/decisions/) | Why the repository is shaped the way it is, including the ones that turned out wrong |
| [`crates/`](crates/) | The domain computer: 12 Rust crates, 617 test functions, no crates.io dependencies |
| [`web/`](web/) | The site — a page per district, statewide views, and a scenario runner, all static |
| [`agents/`](agents/), [`.yidam/skills/`](.yidam/skills/) | The traversals and procedures this domain keeps needing |

**The corpus holds mechanism; the crates hold the numbers.** Roughly 610 education agencies across
the years with usable data would drown a graph, so per-agency series live as committed CSV under
`crates/` and a corpus node cites a series rather than restating it.

## The two halves, and why the split matters

**Retrieval can fail. Calculation cannot.** [`connect`](crates/connect/) and
[`spreadsheet`](crates/spreadsheet/) fetch and parse what Ohio and the federal government publish;
nothing in them computes a funding figure. The nine calculators are pure and deterministic and
read only committed fixtures.

That separation is what makes the whole thing reproducible offline. Every fixture is committed,
every one of them rebuilds byte-identically from a clean checkout, and the 43 published files they
come from are pinned by SHA-256 — because the department reposts corrected workbooks at the same
URL, so "regenerated from the FY2027 calculator" otherwise names a moving target.

13 connectors, 11 wired. The two that are not are blocked on things this repository cannot build:
an authenticated portal, and a publisher that serves interactive maps to a browser and a 404 to
anything that identifies itself. Each says which, in a field a test checks —
[the table](crates/connect/README.md) is generated from the registry rather than written by hand.

## Running it

Tasks are [mise](https://mise.jdx.dev) tasks. `mise install` provisions Rust, Node and pnpm;
`mise tasks --all` lists everything.

```
mise run //:ci             everything the CI workflow runs, in the same order
mise run //crates:gate     fmt, clippy, test, doc
mise run //web:gate        check, unit, build, e2e
mise run //:generated      fail if the feed or any README block is stale
mise run //crates:connectors   what is retrievable, and how far each connector got
```

`.github/workflows/ci.yml` runs the same gates and has **never executed**, because this repository
has no remote. That is deliberate; `//:ci` is the thing that runs, and the workflow is kept
unchanged so it works the day a remote is added. The cost of the gap is on the record: `cargo doc`
with warnings as errors was failing for at least two phases and nothing said so.

If `cargo` is not found outside mise, the host toolchain is at `~/.cargo/bin`.

## How to read a claim

Every non-obvious statement in the corpus carries a tag, and they are counted rather than
asserted — 447 `[verified]`, 171 `[inference]`, 128 `[open]`, 25 `[unentered]`.

- `[verified]` — a committed primary source backs it
- `[inference]` — drawn from verified facts, not witnessed. Not a weakness; untagged inference is
- `[open]` — a live question. Somebody has to find out
- `[unentered]` — a knowable value nobody has typed in. A [local narrowing](.yidam/corpus/README.md)
  of `[open]`, because a corpus reporting 153 open questions reads as one with deep uncertainty
  about its domain when a sixth of them are empty fields

## What this does not model

Stated plainly because the alternative is a reader assuming otherwise:

- **The scholarship and community-school channel.** The 609 districts everywhere here are the
  traditional districts in the department's own calculator. A district's foundation payment can
  rise while its net position worsens, and nothing in this repository would show it.
- **Anything before FY2020.** The declared scope is 1851 to the present. Every committed fixture is
  FY2020 or later. The regimes are documented; the per-district figures behind them are not.
- **The capital channel.** Facilities assistance is invisible in every operating per-pupil figure,
  and was itself part of the *DeRolph* remedy.

## Provenance

Sources are Ohio's Department of Education and Workforce, the Department of Taxation, the
Legislative Service Commission, the General Assembly and Supreme Court archives, and — for
comparability and for identifier stability across agency mergers — the Census Bureau, NCES, and
the Bureau of Labor Statistics. Every one has a [catalog record](.yidam/catalog/) saying what it
can be trusted for, because a foundation payment report, an LSC estimate made before the year
closed, and a district's own five-year forecast are three numbers describing the same thing that
routinely disagree.
