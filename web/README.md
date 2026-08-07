# web

The interface layer. School boards, journalists, legislative staff, and parents will not be
running a Rust CLI, so this was in scope from the outset.

Three views:

- **District** — what the Fair School Funding Plan computes for one district, what it actually
  receives, and where the difference comes from.
- **Statewide** — who is on the guarantee, whether state aid offsets property wealth, and the
  two floors.
- **Outcomes** — what districts achieve, and how little of it the funding side explains.
- **Scenario** — move a lever and see who it reaches, across all 609 districts, in the browser.

Every view is linkable: `#district/043786`, `#statewide`,
`#scenario?g=phase-out&arg=0.5&base=1.05`. A scenario worth arguing about is worth being able
to send someone.

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
pnpm --dir web check        # types, including the .astro file
pnpm --dir web test:unit    # vitest — the formula, against the committed feed
pnpm --dir web test:e2e     # playwright — the page in Chromium, against a real build
pnpm --dir web test         # both
pnpm --dir web build        # writes dist/
```

The split is the point. **Vitest** runs the formula: no browser, no build, ~200ms, and it is
where a drift between this implementation and the Rust shows up. It goes through
[Astro's own Vite config](vitest.config.ts) so the modules resolve exactly as the shipped bundle
resolves them — a test passing under different resolution rules than the browser gets is testing
something adjacent to the thing that ships.

**Playwright** builds the site and serves `dist/` on a fixed port of its own, deliberately not
4321 — a suite that silently tests whatever answered is worse than one that cannot start.

## The build

[Astro](https://astro.build) in static mode, over Vite. `pnpm build` writes [`dist/`](dist/):
one HTML document, one hashed JavaScript bundle, one hashed stylesheet, and the feed. Nothing
runs behind it — that is the property static hosting wants, and the one the previous hand-rolled
`tsc` build had.

What changed is where the output lives. `app/` was committed `tsc` output, so that the
deployable thing needed no build step at the far end. `dist/` is **not** committed: reproducing
it is one command now, and a content-hashed bundle in the history is churn. `src/` is still the
source of truth, and is still the only thing to edit.

[`public/data/bundle.json`](public/data/) is deliberately outside the module graph. It is
fetched at runtime rather than imported, so regenerating the feed stays a `cargo run` redirect
and does not require rebuilding the site. Everything else — nine TypeScript modules that used to
be nine network requests — Vite bundles and preloads.

```
src/pages/index.astro   the document; markup only, deliberately inert
src/scripts/main.ts     the client entry — routing, wiring, the verification gate
src/lib/*.ts            the formula, the views, the charts, the formatting
src/styles/app.css
public/data/bundle.json the feed, copied verbatim into dist/
tests/unit/             the formula, over the committed feed
tests/e2e/              the page, in Chromium, against a real build
```

## On dependencies

This used to have none, and said so at length. It now has a [`package.json`](package.json) and a
lockfile, and the reason the old position was taken is worth keeping in view rather than quietly
deleting: the extraction pipeline and the calculators have no dependencies because a build that
stops working when a transitive package is yanked is a real failure mode, not a purity concern.

Nothing about that reasoning became wrong. What changed is the other side of the trade. The
three things the old setup hand-rolled were a test runner, a static server, and a slice of
`@types/node` — all cheap. The thing it could not hand-roll was **a real browser**, and this
page's whole safety argument is about what it does or does not put in front of a reader. The
first afternoon of Playwright found the bug below.

Every dependency here is a `devDependency`. Nothing is committed to `dist/` and the deployed
artifact is still inert files with nothing running behind them, so the exposure is build-time
only: if the tree rots, the site that is already deployed does not.

## The scenario tab could show an unverified scenario

The page refuses to render a scenario until its formula reproduces the Rust checkpoints (below).
It got that right at load and then wrote over itself: `reportVerificationFailure()` put the
"disabled" panel into `#scenario-out`, and clicking the **Scenario** tab called `render()`,
which replaced it with a rendered scenario. The warning vanished from the one tab it existed to
hold shut, and what replaced it looked like a working page.

The tell was in the code already — `state.verification` was stored and never read. `render()`
now checks it, and [an end-to-end test](tests/e2e/app.spec.ts) tampers with a checkpoint in
flight and clicks through to the tab, because this is a class of bug that only exists once a
browser has run the code. The twenty tests in [`tests/unit/`](tests/unit/) could not have caught
it, and no number of them could have: the formula was never wrong.

## The scenario builder computes the formula twice, on purpose

[`src/lib/policy.ts`](src/lib/policy.ts) is a second implementation of
`crates/project/src/policy.rs`. Two implementations of one formula is normally a bad trade: they
drift, and the one nobody runs is the one that is wrong.

What makes it acceptable is that **the feed carries Rust-computed checkpoints and the page will
not render a scenario until it reproduces every one of them** — against the real 609-district
panel, on every load, to within a dollar across seven billion. If they disagree the tab is
disabled and the page says which scenario differed and by how much. The Rust stays
authoritative; the TypeScript has to prove it agrees.

This is not theoretical. Wiring it up **found a bug in the Rust**: a district pulled below a
*raised* minimum state share was taking the branch meant for districts already on the floor, and
getting its published share scaled by the ratio of the minimums instead of the floor itself. It
was worth $4.7 million and 273 Rust tests did not catch it, because the wrong answer is still an
increase and still confined to districts below the new floor. The independent implementation
disagreed, and the disagreement was the signal.

## The outcome view exists to prevent two mistakes

Not to display the Performance Index. To make these hard:

**Reading a correlation with poverty as a correlation with something else.** Poverty explains
Ohio's attainment measure at −0.846. The guarantee is the sharpest case: districts it protects
score +0.187 higher, and +0.035 once poverty is held constant. Every association on that tab is
rendered with its controlled figure beside it, and an e2e test fails if one ever appears alone.

**Quoting a per-pupil number without its denominator.** The department divides operating
spending by a need-weighted pupil count. Against a composition-driven outcome that is mostly a
composition proxy — the same numerator gives −0.004 one way and −0.355 the other. Both are in
the feed and both are on the page.

## Design notes

Two series appear anywhere here — formula aid and guarantee, reused as gain and loss — in the
categorical slots validated for both surfaces: `#2a78d6`/`#eb6834` light, `#3987e5`/`#d95926`
dark. **All six checks pass in both modes.** They carry direct labels and a legend as well as
color, so identity never rests on hue. Dark mode is a selected step from the same ramps rather
than an inversion, and `data-theme` beats the OS setting in both directions.

The scenario histogram uses the same two as a **diverging** pair, so zero is neutral gray and
carries a dashed rule and a "no change" label — a reader has to be able to see where zero is,
not infer it from the hues.

Distribution position uses a marker on a neutral strip rather than a chart: the question is
"where does this district sit", which is one value against a range, not a series.

## The projection, and how the interval got to be the subject

The forecast half of `crates/project` was held back for one reason: a forecast rendered as
"$7.13B (±3.4%)" is read as $7.13B. The interval becomes a disclaimer the eye skips, which is
worse than not showing one, because it lends a forecast the authority of a measurement. The
design answer this settled on is to **invert which of the two is the headline**:

- The tile's large number is the **range** — `$6.89B – $7.38B`. The central estimate is the
  small line underneath, labelled as one path through the band rather than the answer. Every
  other tile on this page leads with a point; this one deliberately does not.
- In the fan chart the band is a **mark**, not shading: a fill with a 2px stroke on each edge.
  The centre line is **dashed**, because a solid one reads as the answer with error bars around
  it. Only the two bounds are direct-labelled.
- The y axis is truncated to the band's own range — against a zero baseline a band a few percent
  wide is invisible — so it says `axis starts at …, not zero` on its face.

The forecast card is always **below** the simulation and is never merged with it, and there is
deliberately no figure anywhere that adds them together: re-running the formula with a lever
moved is exact, projecting enrollment six years out is not, and a combined number inherits the
second's error while wearing the first's precision.

The page carries its own copy of the projection so the horizon slider needs no round trip — the
same trade as `policy.ts`, held to the same standard. The feed carries Rust-computed
`ForecastCheckpoint`s at four (policy, year) pairs and the page reproduces every one, **at both
ends of the band as well as the middle**, before it will draw anything. A page that reproduced
the point and got the interval wrong would look exactly right. The two gates are separate: a
failed forecast check costs the reader the band, not the scenario builder.

On the district view the same machinery runs over one district instead of 606. For a district
the guarantee pays, the band collapses to a flat line — its aid does not respond to its
enrollment at all — and the caption says so, because that is the finding and not a broken chart.

## What is not built

The [regime view](../.yidam/corpus/funding-regime/) — walking the history from the Foundation
Program to the Fair School Funding Plan — which needs a historical series the corpus does not
have.

**Year-over-year funding.** The district view compares the FY2027 formula run at each of three
enrollment years, which isolates the enrollment channel exactly. It is not a comparison of
published FY2026 and FY2027 totals, and it says so on the card: the department publishes one
foundation calculator at a time and replaces rather than archives it, so no FY2026 per-district
payment figure exists in this repository to compare against. Getting one is a
[`dew-foundation`](../crates/connect/sources/dew-foundation.md) source problem, not a rendering
one.

Deployment target is still undecided; static hosting with this bundle is the presumption.

## Bundle status

<!-- REGEN: yidam bundle-status
Regenerated by: `yidam bundle-status`
Fields: bundle contract version, feed list, last export timestamp, node counts per feed,
        deployment target, last deploy status.
-->
| Field | Value |
|---|---|
| Contract version | `5.0.0` |
| Districts in the feed | 609 |
| Reference checkpoints | 8 |
| Reference forecasts | 4 |
| Size | 1110 KB |
| Deployment target | none chosen; static hosting is the presumption |

Regenerate with `cargo run --manifest-path crates/Cargo.toml -p bundle > web/public/data/bundle.json`. CI fails if the committed feed and a fresh one differ.
<!-- /REGEN -->
