# web

The interface layer. School boards, journalists, legislative staff, and parents will not be
running a Rust CLI, so this was in scope from the outset.

Roughly 2,500 pages, all of them static files:

| Route | What it answers |
|---|---|
| `/` | Statewide: who is on the guarantee, whether state aid offsets property wealth, the two floors. |
| `/districts` | All 609, sortable and filterable. |
| `/district/{irn}` | What the Fair School Funding Plan computes for one district, what it actually receives, and where the difference comes from. |
| `/district/{irn}/outcome` | What its pupils achieve — against districts with comparable poverty, never against the state. |
| `/district/{irn}/finances` | What it actually collected and spent. The only figures here that are a record rather than a model. |
| `/district/{irn}/scenario` | What a proposed change does to this district, and how many districts it moves the other way. |
| `/outcomes` | Statewide: how little of attainment the funding side explains. |
| `/scenario` | Move a lever and see who it reaches, across all 609, in the browser. |
| `/compare` | Two districts side by side. |
| `/wiki` | The corpus — regimes, statutes, litigation, parameters, metric definitions — rendered from `.yidam/` directly. |
| `/method` · `/data` · `/search` | How the figures are made, the downloads, and one box over all of it. |

Every one of them is a real URL. `#district/043786` used to be the shareable form and still
redirects, but the address is `/district/043786` now, and it is a document rather than an
instruction to a router.

## Running it

```
cargo run -p bundle --manifest-path crates/Cargo.toml > web/public/data/bundle.json   # regenerate the feed
pnpm --dir web install                                                                # once
pnpm --dir web dev                                                                    # prints its URL
```

Astro's dev server backgrounds itself and takes the first free port, so read the URL it prints
rather than assuming 4321. `pnpm --dir web exec astro dev stop` stops it; `status` says whether
one is running. A stray one from another project is the likeliest reason a page looks wrong.

```
pnpm --dir web check          # types, including the .astro files
pnpm --dir web test:unit      # vitest — the formula, the schemas, the link graph, ~600ms
pnpm --dir web test:e2e       # playwright — the site in Chromium, against a real build
pnpm --dir web test           # both
pnpm --dir web build          # writes dist/
pnpm --dir web schemas        # regenerate .yidam/schemas/ for the editor
pnpm --dir web corpus:report  # every corpus diagnostic, grouped
```

The split is the point. **Vitest** runs the formula: no browser, no build, and it is where a drift
between this implementation and the Rust shows up. It also runs the link checker — every route the
wiki generates from a relative corpus path is resolved against the set of pages the build actually
emits, because a rewritten link that 404s looks exactly like one that works.

**Playwright** builds the site and serves `dist/` on a fixed port of its own, deliberately not
4321 — a suite that silently tests whatever answered is worse than one that cannot start. A third
of it runs with **JavaScript disabled**, which is the property the whole page-per-district
architecture exists to provide.

## The build, and what it gave up

[Astro](https://astro.build) in static mode over Vite. `pnpm build` writes [`dist/`](dist/): about
2,500 documents, one hashed stylesheet, a code-split JavaScript bundle, and the data files.
Nothing runs behind it.

This used to be **one** document that fetched a 1.1 MB feed and rendered every figure in the
browser. The feed sat outside the module graph on purpose, so regenerating it was a `cargo run`
redirect and never a rebuild — and that property is the thing this architecture spent to get real
routes. A district's numbers are now written into that district's HTML at build time, so **the
feed and the build are one artefact and publishing a feed change is a rebuild**.

What that bought:

- A district page is **12 KB**, 4 KB over the wire, against 1.1 MB before.
- Every page is complete before any script runs. Figures, tables, and charts are all in the
  document — the charts as SVG, not as a canvas drawn on load.
- A search engine, a screen reader, and a text browser all get the whole page.
- 2,500 pages build in about 5 seconds, so the trade costs nothing in practice.

The feed is still copied verbatim into `dist/` and still served. It is what the scenario routes
fetch, and what [`/data`](src/pages/data.astro) offers for download.

## The verification gate moved to build time

[`src/lib/policy.ts`](src/lib/policy.ts) is a second implementation of
`crates/project/src/policy.rs`. Two implementations of one formula is normally a bad trade: they
drift, and the one nobody runs is the one that is wrong.

What makes it acceptable is that **the feed carries Rust-computed checkpoints and the TypeScript
has to reproduce every one of them** — against the real 609-district panel, to within a dollar
across seven billion — before anything is rendered. That check used to run on every page load and
disable the scenario tab when it failed. Disabling a tab is the right answer when the numbers
arrive after the page does; it is the wrong answer once the figures are baked, because by then
they are already printed.

So [`loadFeed()`](src/lib/feed.ts) runs it and **throws**, which fails the build. A drifted
formula cannot be deployed at all rather than being deployed with one tab held shut. That is
strictly stronger than what it replaced.

It still runs in the browser on the two scenario routes, because those genuinely compute there and
because the panel is a separate file that can be replaced without a rebuild. A gate that only ever
ran where the answer was already known would be decorative — and
[an end-to-end test](tests/e2e/app.spec.ts) tampers with a checkpoint in flight to prove it is not.

The two halves stay separate, as before: a failed **forecast** check costs the reader the band and
nothing else, and the build succeeds with a warning. They are different claims and one can be
wrong alone.

This is not theoretical. Wiring it up originally **found a bug in the Rust**: a district pulled
below a *raised* minimum state share was taking the branch meant for districts already on the
floor, and getting its published share scaled by the ratio of the minimums instead of the floor
itself. It was worth $4.7 million and 273 Rust tests did not catch it, because the wrong answer is
still an increase and still confined to districts below the new floor.

## Both inputs have a schema, and both stop the build

This site reads two things it does not own: a JSON feed written by `crates/bundle`, and 75 YAML
files written by hand. Until recently it validated neither — the feed was `JSON.parse(raw) as
Bundle`, and the corpus was read field by field with `String(x ?? "")`. Both are now parsed against
a declared shape, and both fail the build when they do not match.

### The feed

[`src/lib/schema/feed.ts`](src/lib/schema/feed.ts) declares the bundle once, in zod, and the
TypeScript types are inferred from it — so there is one definition rather than an interface and a
validator that can disagree. `types.ts` re-exports those types with `export type`, which TypeScript
erases, which is how the browser gets `Panel` without getting zod.

A cast is not a check. If the Rust renamed a field, nothing anywhere noticed: the feed parsed, the
types agreed, and 609 pages rendered `undefined` — formatted as an em dash, which reads as "not
reported" rather than as a defect. Every object is `.strict()`, so a field appearing that this
mirror does not know about fails too. That sounds unhelpful and is the point: this file is a
hand-maintained mirror of a Rust struct, and the only failure that matters is the two drifting
apart. The contract version catches deliberate breaks; strictness catches accidental ones.

The Rust is still authoritative. This proves the *shape* agrees; the checkpoints prove the
*arithmetic* does, and CI diffs the committed feed against a freshly generated one.

### The corpus

[`src/lib/schema/corpus.ts`](src/lib/schema/corpus.ts) does the same for nodes and ontology
classes, plus the cross-file checks a per-file schema cannot make: that every node is an instance
of a class that exists, that every link target resolves to something, and that every target is a
shape this site can turn into a URL.

The line between an error and a warning is drawn on measurement, not taste. **94% of properties**
and only **68% of relationships** are declared in their class's own ontology. The property
vocabulary is effectively closed; the relationship vocabulary plainly is not — ninety-odd distinct
relationships, most used once, describing genuinely different connections. Enforcing the ontology's
list would reject a third of the corpus's edges. So structure is an error and vocabulary is a
warning: 93 of them, listed by `pnpm corpus:report`.

That asymmetry is worth a decision at some point. If the relationship vocabulary is meant to be
open, the `edges:` in an ontology file are documentation rather than a constraint, and it would be
worth saying so there.

### And in the editor, which is where it actually helps

`pnpm schemas` writes the corpus schemas to [`.yidam/schemas/`](../.yidam/schemas/) as JSON Schema,
emitted from the same zod definitions by zod 4. [`.vscode/settings.json`](../.vscode/settings.json)
points `yaml-language-server` at them, so a node written wrong is underlined as it is typed rather
than found days later by a page rendering strangely — which is how all four of this corpus's
authoring defects were actually found. The committed copies are checked for staleness by the unit
suite, like any other generated file here.

Neither schema costs a new dependency: `astro/zod` is already in the tree, and zod 4 emits JSON
Schema on its own.

## Charts are Observable Plot, rendered at build time

[`src/lib/plot/`](src/lib/plot/) holds three chart forms as Plot specifications, and two renderers
that share them: [`ssr.ts`](src/lib/plot/ssr.ts) draws into a `linkedom` document during the build,
and [`client.ts`](src/lib/plot/client.ts) draws into the browser's own. One description of what a
chart looks like, so the interactive copy cannot drift from the static one.

Almost everything takes the first path, which means **Plot is a devDependency for every page
except the two scenario routes**. The reader downloads no charting library, and a chart is present
with scripting off.

The consequence is that a build-time chart cannot re-render when the reader switches theme, so
**every colour handed to Plot is a `var(--…)` reference** resolved by the browser against the same
custom properties as the rest of the stylesheet. `var()` works in an SVG presentation attribute and
inherits normally, which is what makes this possible; `ensureThemeable` fails the build if a
literal colour ever slips through, because the resulting bug is invisible to whoever writes it —
they are looking at light mode, where the literal is very nearly right.

## Design notes

Two series appear anywhere here — formula aid and guarantee, reused as gain and loss — in the
categorical slots validated for both surfaces: `#2a78d6`/`#eb6834` light, `#3987e5`/`#d95926` dark.
**All six checks pass in both modes** (worst adjacent-pair CVD separation ΔE 24.7 protan). They
carry direct labels and a legend as well as colour, so identity never rests on hue. Dark mode is a
selected step from the same ramps rather than an inversion, and `data-theme` beats the OS setting
in both directions.

The scenario histogram uses the same two as a **diverging** pair, so zero is neutral gray and
carries a dashed rule and a "no change" label — a reader has to be able to see where zero is, not
infer it from the hues.

Distribution position uses a marker on a neutral strip rather than a chart: the question is "where
does this district sit", which is one value against a range, not a series.

The nominal/constant-dollar switch is **two radio inputs and a sibling selector**, with no script
behind it. Both panels are rendered at build and CSS decides which is shown. The previous
implementation re-rendered from a click handler, which meant a reader without JavaScript got the
nominal view and a dead button.

## The projection, and how the interval got to be the subject

The forecast half of `crates/project` was held back for one reason: a forecast rendered as
"$7.13B (±3.4%)" is read as $7.13B. The interval becomes a disclaimer the eye skips, which is worse
than not showing one, because it lends a forecast the authority of a measurement. The design answer
is to **invert which of the two is the headline**:

- The tile's large number is the **range** — `$6.30B – $6.90B`. The central estimate is the small
  line underneath, labelled as one path through the band rather than the answer.
- In the fan chart the band is a **mark**, not shading: a fill with a 2px stroke on each edge. The
  centre line is **dashed**, because a solid one reads as the answer with error bars around it.
  Only the two bounds are direct-labelled.
- The y axis is truncated to the band's own range — against a zero baseline a band a few percent
  wide is invisible — so it says `axis starts at …, not zero` on its face.

The forecast card is always **below** the simulation and is never merged with it, and there is
deliberately no figure anywhere that adds them together: re-running the formula with a lever moved
is exact, projecting enrollment six years out is not, and a combined number inherits the second's
error while wearing the first's precision.

On a district page the same machinery runs over one district instead of 606. For a district the
guarantee pays, the band collapses to a flat line — its aid does not respond to its enrollment at
all — and the caption says so, because that is the finding and not a broken chart.

## The outcome routes exist to prevent two mistakes

Not to display the Performance Index. To make these hard:

**Reading a correlation with poverty as a correlation with something else.** Poverty explains
Ohio's attainment measure at −0.846. The guarantee is the sharpest case: districts it protects
score +0.187 higher, and +0.035 once poverty is held constant. Every association is rendered with
its controlled figure beside it, and an e2e test fails if one ever appears alone.

**Quoting a per-pupil number without its denominator.** The department divides operating spending
by a need-weighted pupil count. Against a composition-driven outcome that is mostly a composition
proxy — the same numerator gives −0.004 one way and −0.355 the other. Both are in the feed and both
are on the page.

The per-district outcome route applies the same discipline to a single number: a district's
Performance Index is shown against the median of its **poverty fifth**, not against the state,
because comparing it to the state median is mostly comparing its poverty rate to the state's.

## The wiki reads `.yidam/` directly

[`src/lib/corpus.ts`](src/lib/corpus.ts) parses the corpus YAML and the catalog markdown at build
time — no Rust export crate, no committed intermediate. Numbers go through `crates/bundle` because
numbers are computed and Rust is authoritative for the computation; the corpus is a document graph
and nothing decides anything along the way, so a second serialization would only be a thing to keep
in sync.

Two things a naive reader of that corpus gets wrong, both found by building this:

**Links are written in four places, not one.** `links:` is the structured form. But descriptions
are dense with inline citations, two nodes carry a `findings:` block, property values contain
links, and one node — `scenario/guarantee-phase-out` — writes its entire link list as a *prose
paragraph*. That last one is valid YAML, so nothing complains, and a graph built only from `links:`
silently drops all fifteen of its edges. The reader takes all four, and marks which edges the
author declared versus merely mentioned.

**Claims carry their epistemic status inline.** `[verified]`, `[inference]` and `[open]`, sometimes
with a justification attached. That convention is the reason the corpus is worth reading, and
markdown renders it as stray brackets, so it becomes badges instead.

Relative file paths — `../parameter/twenty-mill-floor.yml` — are rewritten to routes before the
markdown processor runs. Roughly 200 of them, each a small guess about a path shape, which is why
[`tests/unit/links.spec.ts`](tests/unit/links.spec.ts) resolves every one against the pages the
build actually emits.

Where a corpus node names a district with an IRN in the feed — five of the seven exemplar agencies
— the two are joined in both directions: the corpus says what the district illustrates, the
district page says what it is currently paid.

### Three corpus defects this surfaced

Building the wiki required parsing all 62 nodes, which nothing had done before. Three were not
parseable or not correct, and are worth fixing at the source rather than working around:

1. **Two nodes were invalid YAML.** `toledo-city` and `perrysburg-exempted-village` wrote
   `irn: "044909" [verified — …]`; a quoted scalar followed by a bracket is a syntax error. Fixed
   here to the unquoted form the other five agency nodes already use.
2. **Four nodes had a plain scalar containing `: `.** `series_path: Not yet populated. [open]
   Target: crates/ data files…` parses as a nested mapping and fails. Converted to block scalars,
   preserving the text exactly.
3. **One node writes `links:` as prose.** `scenario/guarantee-phase-out` — described above.
   *Not* fixed here, because converting it means choosing relationship slugs and that is an
   authoring decision. The wiki renders it correctly and recovers its links, and the node's page
   says why it has no relationship names.

## Deploying

The site is served at **<https://schools.ohio.shawneesmart.systems>** from Cloudflare Pages,
project `ohio-education-funding`. There is no deploy job: publishing is a thing someone does on
purpose.

From the repository root:

```
pnpm --dir web test                       # both suites
pnpm --dir web build
pnpm dlx wrangler@latest pages deploy web/dist \
  --project-name ohio-education-funding --branch main
```

Two things about that third line are easy to get wrong and quiet when you do. `--dir` steers which
package pnpm runs, not where `dlx` resolves paths, so the directory argument is `web/dist` and not
`dist` — passed the latter from the root, wrangler goes looking at the repository root and exits
`ENOENT`. And `--branch main` is not decoration: on direct upload wrangler labels the deployment
with whatever branch is checked out, and only the production branch reaches the production URL.
Deploy from a `phase/*` branch without it and the upload succeeds, wrangler prints a URL, and the
live site does not change — you have published a preview.

**A feed change is now a rebuild.** `cargo run -p bundle` followed by the three commands above,
in that order. The old shortcut of regenerating `public/data/bundle.json` alone no longer updates
what a reader sees, because the figures are in the HTML.

`dist/` is about 35 MB across ~2,550 files, well inside Cloudflare Pages' 20,000-file limit.

DNS is split, and knowing which half is which saves an hour when something breaks.
`shawneesmart.systems` is authoritative on **Route 53**, and stays there — it carries the mail
records. The only thing Cloudflare owns is one `CNAME` at `schools.ohio` pointing to
`ohio-education-funding.pages.dev`, plus the certificate it issues for that hostname. Cloudflare
Pages custom domains work against external DNS this way on the free plan; a Workers deployment
would not, because a Workers custom domain requires the whole zone to live on Cloudflare.

The order that setup happens in matters and is not recoverable in the moment: the hostname must be
registered on the Pages project **before** the `CNAME` exists in Route 53. Reversed, the record
resolves to a Pages edge that has never heard of the hostname and answers 522 until the association
catches up.

[`public/_headers`](public/) is the deploy's one piece of configuration, and Astro copies it into
`dist/` like any other public asset — Pages reads it from the deploy root rather than serving it.
It pins `_astro/*` for a year, which is safe because Vite content-hashes those names, and
deliberately does not pin the data files, which have fixed paths. Its CSP allows inline *style
attributes* and `<style>` elements because Plot emits both inside its SVG; `script-src` stays
strict, which is the half that matters.

## On dependencies

This had none for a long time, and the reason is worth keeping in view: the extraction pipeline and
the calculators have no dependencies because a build that stops working when a transitive package
is yanked is a real failure mode, not a purity concern.

Nothing about that became wrong. What changed is the other side of the trade — the things that
cannot be hand-rolled. A real browser, which is what caught the verification-gate bug. A markdown
processor, for a corpus that is markdown. And Observable Plot, which is a chart grammar rather than
a chart library and would be several thousand lines to reproduce badly.

Every dependency here is a `devDependency` **except** `@observablehq/plot`, and it is code-split so
it ships only on the two scenario routes. The deployed artifact is still inert files with nothing
running behind them, so the exposure is build-time only: if the tree rots, the site that is already
deployed does not.

## Layout

```
src/pages/            one file per route; the dynamic ones call getStaticPaths over the feed
src/layouts/          the shell, and the district sub-navigation
src/components/       the basis switch, the lever controls, the district nav
src/lib/feed.ts       the feed, read at build, parsed, indexed — throws to fail the build
src/lib/corpus.ts     the corpus, read at build: nodes, ontology, sources, backlinks
src/lib/schema/       what a feed and a corpus file are allowed to be; the only definition of each
src/lib/prose.ts      corpus markdown: link rewriting and claim badges
scripts/              schema emission and the corpus report; run by pnpm, not by the build
src/lib/plot/         chart specifications, and the two renderers that share them
src/lib/*.ts          the formula, the views, the formatting
src/scripts/          the client entries; small, and never load-bearing
public/data/          the feed, copied verbatim into dist/
public/_headers       cache and security headers, read by the host
tests/unit/           the formula, the projection, the link graph, the 404 matcher
tests/e2e/            the site in Chromium, a third of it with JavaScript disabled
```

## What is not built

The **regime view** — walking the numbers from the Foundation Program to the Fair School Funding
Plan — which needs a historical series the corpus does not have. The regimes are documented in the
wiki; the per-district figures behind them are not.

**Year-over-year funding.** The district view compares the FY2027 formula run at each of three
enrollment years, which isolates the enrollment channel exactly. It is not a comparison of
published FY2026 and FY2027 totals, and it says so on the card: the department publishes one
foundation calculator at a time and replaces rather than archives it, so no FY2026 per-district
payment figure exists in this repository. Getting one is a
[`dew-foundation`](../crates/connect/sources/dew-foundation.md) source problem, not a rendering one.

**The community school and scholarship deduction.** Not modelled; the 609 districts here are the
traditional districts in the state's own calculator.

## Bundle status

<!-- REGEN: yidam bundle-status
Regenerated by: `yidam bundle-status`
Fields: bundle contract version, feed list, last export timestamp, node counts per feed,
        deployment target, last deploy status.
-->
| Field | Value |
|---|---|
| Contract version | `6.0.0` |
| Districts in the feed | 609 |
| Reference checkpoints | 8 |
| Reference forecasts | 4 |
| Size | 1111 KB |
| Deployment target | none chosen; static hosting is the presumption |

Regenerate with `cargo run --manifest-path crates/Cargo.toml -p bundle > web/public/data/bundle.json`. CI fails if the committed feed and a fresh one differ.
<!-- /REGEN -->
