# web

The interface layer. School boards, journalists, legislative staff, and parents will not be
running a Rust CLI, so this was in scope from the outset.

Roughly 3,500 pages, all of them static files:

| Route | What it answers |
|---|---|
| `/` | The front door: what this is, how to find a district, and the five sections. Not a view of anything. |
| `/statewide` | Who is on the guarantee, whether state aid offsets property wealth, the two floors. |
| `/districts` | All 609, sortable and filterable. |
| `/district/{irn}` | What the Fair School Funding Plan computes for one district, what it actually receives, and where the difference comes from. |
| `/district/{irn}/outcome` | What its pupils achieve — against districts with comparable poverty, never against the state. |
| `/district/{irn}/finances` | What it actually collected and spent, and what it spent it on by function. |
| `/district/{irn}/taxes` | What property here is worth by class, what is charged on it, and which side of the 20-mill floor that puts the district on. |
| `/district/{irn}/scenario` | What a proposed change does to this district, and how many districts it moves the other way. |
| `/outcomes` | Statewide: how little of attainment the funding side explains. |
| `/history` | FY2009–FY2022 on the federal survey: where the money came from, and whom it reached. The only route that reaches before FY2020. |
| `/scenario` | Move a lever and see who it reaches, across all 609, in the browser. |
| `/compare` | Two districts side by side. |
| `/legislation` | Every act behind the formula, in the order it was signed: five regimes across fifty fiscal years, what each act did, and which biennium it paid for. Generated from the corpus. |
| `/wiki` | The corpus — regimes, statutes, litigation, parameters, metric definitions — rendered from `.yidam/` directly. |
| `/method` · `/data` · `/search` | How the figures are made, the downloads, and one box over all of it. |

Every one of them is a real URL. `#district/043786` used to be the shareable form and still
redirects, but the address is `/district/043786` now, and it is a document rather than an
instruction to a router.

The bar carries five axes, and every one of them is a `<details>` disclosure:

    Places▾    Law▾    Formula▾    Research▾    Reference▾

**Two of them are lifted out of `.yidam/corpus/`.** `Law` reaches the acts, the cases and the
doctrines; `Formula` reaches the regimes and the components, parameters and metrics they are built
from. Before this they were behind `Reference › Wiki` and a scan of eighteen class names — three
clicks, from the section a reader looks at last. Which seven of the sixteen acts the `Law` panel
names is *derived* rather than typed: an act appears if it establishes a funding regime, if it
does not appropriate at all, or if it is the most recently signed act that does. The menu
re-points itself at the next budget with no edit anywhere. `src/lib/nav.ts` computes the bar and
`Base.astro` renders what it is given; `.yidam/decisions/the-bar-lifts-the-corpus.yml` records why.
The panel's first entry is `/legislation`, which is the one thing in it that is not a node: the
same graph read as a chronology rather than as a class.

Disclosures are native ones, because a third of the end-to-end suite runs with JavaScript
disabled — and every destination in the bar is now inside a group, so a scripted menu would put
the whole site behind a script on a site whose whole claim is that none of it is. `site.ts` adds
the closing half: one menu closes another, Escape closes the open one, a click outside closes it.
Without it a reader gets two menus open at once, which is untidy and never broken.

The cost is that nothing in the bar is one click away any more. The search field beside it is the
one-click path, and it is the path most readers want.

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

[Astro](https://astro.build) in static mode over Vite. `pnpm build` writes [`dist/`](dist/): 3,487
documents, one hashed stylesheet, a code-split JavaScript bundle, and the data files. Nothing runs
behind it.

This used to be **one** document that fetched a 1.1 MB feed and rendered every figure in the
browser. The feed sat outside the module graph on purpose, so regenerating it was a `cargo run`
redirect and never a rebuild — and that property is the thing this architecture spent to get real
routes. A district's numbers are now written into that district's HTML at build time, so **the
feed and the build are one artefact and publishing a feed change is a rebuild**.

What that bought:

- A district page is **99.7 KB**, 16.4 KB over the wire under brotli — against 1.1 MB before, and
  against the 12 KB this line claimed for long enough to be quoted back. The old figure was never
  right; the page has also grown since, most recently by the second copy of every chart that makes
  it legible on a phone. What survives the correction is the comparison it was making: the feed
  was 1.1 MB before a reader saw a single figure.
- Every page is complete before any script runs. Figures, tables, and charts are all in the
  document — the charts as SVG, not as a canvas drawn on load. `/compare` was the last exception
  and is not one any more: it ships the pair it is seeded with, and a reader who changes the pair
  fetches **two files of about a kilobyte** rather than the 641 KB panel. Measured at
  400 Kbps / 400 ms RTT, the table went from **4,340 ms** to 1,688 ms with a query and to *first
  paint, with nothing fetched at all*, without one.
- A search engine, a screen reader, and a text browser all get the whole page.
- 3,487 pages build in about 54 seconds, on eight cores. Not the 5 seconds this line used to
  claim, and worth knowing before reaching for a rebuild: page emission is nearly all of it, three
  quarters of that is the 3,045 district routes, and 15 seconds is rasterising preview cards.
  Astro's `build.concurrency` does not help — the work is synchronous, so raising it past 1 buys
  scheduling overhead and a slower build. Measured, not assumed.

**The stored theme is applied before first paint on a fast connection and 50 ms after it on a slow
one**, which is the last figure #111 asked about and is recorded here rather than acted on. The
issue described "two serial round trips", and that was true: the page loaded one module whose first
line imported another. The `modulepreload` pass in `astro.config.mjs` collapsed that — measured,
`chart.js` and the chrome module both start at 15 ms and finish at 20 and 21 ms, in parallel — so
what is left is parse time and not a fetch. Unthrottled the theme lands at 21 ms against a first
paint of 32 ms, which is no flash at all; at 400 Kbps / 400 ms RTT it lands at 926 ms against 876
ms, which is 50 ms of one. Splitting the theme into its own chunk would still be correct and would
buy that 50 ms; the reason given for it — a round trip — no longer exists.

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

### Every share in the feed is a fraction, and now something checks

The bundle inherits units from its sources rather than normalising them, and for three fields that
went wrong quietly. The report card publishes its shares as **0 to 100**; the profile report
publishes its as a fraction; both were passed straight through. So
`outcome.economically_disadvantaged` and `District.economically_disadvantaged` sat in one document
under the same name, **100× apart**, both typed `maybeNum`, and neither said which it was.

It never reached a page — nothing in `web/` renders the report card's copy — which is the only
reason it was latent rather than a `10000%` on a card. Contract **`35.0.0`** converts the three at
the seam in `crates/bundle`, where every consumer gets it right by default instead of by
remembering, including whoever downloads the feed from `/data`.

The rule is now a test in both layers, because a convention that held for every field but three and
was enforced by nothing is how this happened. Rust asserts it over the bundle it builds; the unit
suite asserts it over the bundle that is *committed*, which is what the site renders and what `/data`
serves. Verified by reintroducing the defect and watching both fail — 606 offenders, named by field
and IRN.

One threshold in that test is set from the data rather than from the common case: each share must be
present on at least 250 districts before its range means anything, because the report card
suppresses small subgroups and `outcome.english_learner` is genuinely on 303 of 609. A floor set
from the other two would have failed on a property of the source rather than on a defect.

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

### One thing here is computed, and it is the base cost

Everything per-district on this site is the department's published model passed through — except
base cost. R.C. 3317.011 assembles it from statutory staffing ratios applied to a district's own
grade-band enrollment, priced at statewide average salaries, and
[`crates/foundation`](../crates/foundation/) runs that statute: twenty-two elements in five
sub-components, all of them in the feed and all of them on the district page.

That is worth doing because "base cost per pupil" is the figure every Ohio funding argument turns
on, and as a single number there is nothing in it to agree or disagree with. Broken up it becomes a
series of decisions — a teacher for every 23 pupils in grades 1–3, a superintendent priced on a ramp
between $80,000 and $160,000, building operation at a flat rate per pupil — which is what a school
board is actually arguing about.

The claim is reconciled rather than asserted. The department publishes its own aggregate for every
district; the card prints it alongside and states the difference. Worst across all 609 is **$1.09**
on figures in the millions, from twenty-two elements each rounded where the department rounds them,
and the residuals cancel to nothing statewide. Proved over the whole panel in
[`crates/foundation/tests/department_model_fy27.rs`](../crates/foundation/tests/department_model_fy27.rs).

`PanelDistrict` omits the build-up, so the twenty-nine numbers per district it adds to the feed
never reach the one download that happens in a browser.

### The local half, and where the money goes

Two committed datasets reached no reader for a long time, and both are now in the feed.

**Table SD-1** is the Ohio Department of Taxation's per-district table: taxable value split into
the classes that are reduced separately, the tax charged on each, and effective millage — for
**two tax years**. Two, because H.B. 920 is a mechanism that only exists as a change. Its reduction
factors roll an effective rate back as valuation rises and cannot roll it below twenty mills, so a
reappraisal does opposite things either side of that floor. With TY2023 and TY2024 that is
countable rather than assertable: **304 of the 439 districts above the floor saw their effective
Class I rate fall, against 4 of the 170 at it**, and 98.7% of every rate reduction in Ohio happens
above the floor.

It is also the only place two arms of the state describe the same district independently. Taxation
and Education agree: SD-1's effective Class I rate matches the District Profile Report's for all
606 districts carrying both, to 0.01 mills, and a unit test holds them to it.

**Spending by function** is the report card's FY2025 operating expenditure split eleven ways. It
sits on the finances route beside the audited actuals and is deliberately *not* merged with them:
different source, different basis, per-pupil against totals, and not summable. The denominator
warning appears here for the third time — these divide by unweighted ADM, and the department's own
headline figure does not.

Every statewide figure either of those views prints — the median charge share, the count of
districts charged more than they spend, the floor split — is **derived in `feed.ts` at build**
rather than written into the copy. That is not fastidiousness: the first draft said two districts
are charged more than they spend, and the answer is three. The third is Mayfield City, whose 103%
the Rust suite already shows is about seven points of levy timing rather than tax burden.

### The corpus

[`src/lib/schema/corpus.ts`](src/lib/schema/corpus.ts) does the same for nodes and ontology
classes, plus the cross-file checks a per-file schema cannot make: that every node is an instance
of a class that exists, that every link target resolves to something, and that every target is a
shape this site can turn into a URL.

The line between an error and a warning was drawn on measurement, not taste — and measuring it
answered a question the corpus had never stated. **94% of properties** were declared in their
class's own ontology; only **68% of relationships** were. The property vocabulary is effectively
closed. The relationship vocabulary plainly is not: 90 distinct relationships against 65 declared,
and the undeclared 46 are almost all single-use precise verbs — `recovered-funds-from`,
`retreats-from`, `reframes` — that say something a generic edge would lose.

So each class now says which it means. `edge_policy: characteristic`, which all 13 currently are,
means `edges:` documents what the class is *defined by* rather than bounding what may be said about
it. `exhaustive` closes the vocabulary and makes anything outside it an error. It is required with
no default, so a class nobody has decided about cannot pass silently.

The property warnings turned out to be nine names that split cleanly: four genuine omissions from
the ontologies — `boundary_note`, `series`, `results`, `procedural_history` — now declared, and
five year-stamped observation snapshots like `fy2024_profile`, which recur but would mean editing
an ontology every fiscal year to permit next year's, and are allowed by naming convention instead.

`pnpm corpus:report` now reports **0 errors and 0 warnings**. That is the point of having stated
the policy: the next undeclared property name is a typo or a decision, rather than one of 93 lines
nobody reads.

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


### The relationships are drawn as relationships

The site carried **ten correlation coefficients and no scatterplot**. A coefficient is one number
standing for six hundred pairs, and two very different clouds produce the same one; the cards it sat
on were the ones whose subject *is* the relationship — "Does state aid offset property wealth?"
answered with two numbers in a table.

`scatterSpec` is the fifth form. One dot per district, 606 of them, on `/` and `/outcomes` only:
those are two pages, where a 606-mark SVG costs nothing, and the same cloud on all 3,045 district
pages would be 22 MB. The per-district version of the question wants a distribution form, not this
one.

Three things about it are deliberate. **The marks break the usual size rule** — 2.4px and partly
transparent, because 8px opaque markers at this density are a solid blob, and density carried as
darkness is the information. The hit target does not shrink with them: it is a separate transparent
7px mark so a reader can point at a district. **Both scales are logarithmic** where the measure is
skewed — valuation per pupil spans 17× and aid per pupil 30×, and on a linear axis nine districts in
ten sit in the left third. **Colour means one thing per chart**: the dots are neutral and the two
median lines carry the hues, because a scheme where hue meant guarantee-status on a dot and series
on a line would be two vocabularies in one frame.

**No line here is fitted.** Every trace is a median per equal-count bin of the x axis — the same
arithmetic `povertyQuintiles` and `guaranteeRateByQuintile` already did for the bar charts, drawn as
a line instead of as five bars. That is the whole reason the traces are allowed in the web layer: a
regression is a claim about a model, and a model belongs in `crates/` with a checkpoint behind it,
while a median describes points the reader can already see. Where a fitted line would say more, the
coefficient beside the chart is the one the crates computed and is stated as what it is.

The quintile bar chart on `/outcomes` is gone, and its five medians are the line through the cloud.
`/` is 31.8 KB → 185.5 KB and `/outcomes` 12.8 KB → 442.3 KB, which is 48.6 KB and 97.0 KB on the
wire; repetitive SVG compresses about 4.5:1.

### The district index shows the distribution it is filtering

609 rows, sortable and filterable, and no sense of the shape you were browsing. The last placement
from the visualisation review, and the only one where the page's own architecture was the question:
filtering happens in the browser, everything else on this site renders at build.

**Six strips rendered at build and one revealed by an attribute selector** — `BasisToggle`'s trick
applied to charts. A reader with no script gets the state-aid distribution and a complete page; a
reader with script gets the strip following whichever column they sorted by, because the sort keys
and the strip keys are the same strings.

The alternative was drawing one in the browser when the sort changes, which would have put
Observable Plot on the district index — **roughly 100 KB gzipped, on a route likely to be someone's
first entry to the site** — for the sake of a 46px strip. All six together cost **4.6 KB gzipped**,
on a page already 52.6 KB from its 609 rows.

Sorting by name leaves the strip where it was rather than blanking it: `name` is the default sort
and is not a quantity.

### The half of the fused-word defect that leaves no evidence

Astro trims a newline between text and an adjacent expression, so a paragraph reflowed across lines
silently loses a space. `7a78b40` found eleven of these live and scanned the built site for the
pattern afterwards — but that scan matches a letter against an inline **tag boundary**, and the
commonest form leaves no tag at all:

```
    ... in the department's FY{bundle.fiscal_year} model.
    {count(s.on_guarantee)} are held above what the formula ...
```

renders `model.294` as one text node. **Six were live** when this was written — `model.294`,
`guarantee;133`, `the$812.5M`, `moves$113.0M`, `about$25.2M`, `,$1.89B` — two of them shipped by
the very commits that documented the defect.

There is no signal in the output to scan for: a letter beside a digit matches **13,441** times
across 400 pages, essentially all of it `textContent` running across a table cell. So this half is
checked at the source, where the defect is an authoring pattern rather than a rendering one.

**The rule is absolute and that is the point.** Every expression opening a line below prose needs an
explicit `{" "}`, whether or not its own value happens to start with a separator. The exception
cannot be checked — telling `{chamber === "senate" ? " Senate seats …"}` from
`{millions(812_500_000).replace("+", "")}` means knowing which literal a conditional will emit — and
the redundancy is free, because **HTML collapses whitespace**. A rule with no false-positive cost
can afford to be blunt, and this defect has shipped seventeen times.

Comments and expressions emitting markup are exempt; neither can fuse to a sentence. Verified by
reintroducing a defect and watching it fail.

### A ratio is one number standing for two

`/counties` ranked its 88 counties by richest ÷ poorest valuation per pupil and printed the ratio.
The two numbers behind it are not recoverable from it: **Auglaize and Harrison are both 1.2× and
$458,326 per pupil apart**, Harrison's *poorest* district standing on more valuation than
Auglaize's richest. Ranking the counties by disparity and ranking them by what their poorest
district has agree for **29 of 84** — the page was showing one of two nearly independent orders.

`rangeSpec` is the seventh form: one row per county, a line from its poorest district to its
richest. The axis is the design. **On a log axis a row's length is its ratio and its position is
its level**, so sorted by ratio the lengths step down the page while the positions scatter freely,
and "the same disparity somewhere else entirely" is the picture rather than a sentence. On a linear
axis the same sort produces lengths with no fixed relation to the number they are sorted by, which
would be worse than the table it sits above. The ends take two shades of one hue, not two hues: a
low end and a high end are one measure at two points.

The county pages gain the other half — where their own disparity sits among the 84, which the card
never said. Cuyahoga is 5.5× against a median county of 2.1×, and only ten counties exceed 4×.

**There is no map, and that is the point.** The obvious county visualisation is a choropleth, and
this page opens by saying a county here is *a peer group, not a boundary* — Ohio school district
lines follow historical township lines and cross county lines freely. Filling a county-shaped
polygon with a funding figure asserts exactly the geography the page spends its first card denying,
and the department's one-county-per-district attribution could not honestly draw the crossing even
if the geometry were here.

### Colour that carries a third variable

A median line says what the middle of a cloud does; colouring the dots by an ordered band says what
the cloud is *made of*. On `/outcomes` that is the card's whole argument, drawn: the three poverty
thirds sit at the **same** spending per need-weighted pupil — tenth percentiles within $500 of each
other, ninetieths within $100 — and at median Performance Indexes eighteen points apart. That is
what a −0.004 correlation looks like from the inside. Switch the denominator to enrolled pupils and
the same three bands separate horizontally too.

**Three bands and not five, and the number is measured.** A scatter is an all-pairs form — any band
can sit beside any other, so every pair has to separate, not just adjacent ones. Five steps of the
formula hue close to a normal-vision ΔE of **10.9**, which is two bands a full-colour reader cannot
tell apart. Three reach 21.4 light and 21.6 dark, 20.8 / 20.2 under the worst CVD simulation. The
ramp is ordinal — one hue, light to dark — because swapping two bands changes the meaning, and it is
a *selected* dark ramp rather than an inversion. Its end steps sit near 2.2:1 against their surface,
which obligates the legend those cards carry.

**Where it is not worth spending.** Where the banding measure is already an axis. Banding the
poverty-against-attainment scatter by poverty repaints the x axis as a gradient and says nothing,
while spending the one channel a third variable could have used; the same is true of the
wealth-neutrality chart, whose two median lines are its finding. And on `/method` a valuation
banding turned out to be floor status in disguise — the terciles' exact-reproduction counts run
80/68/38 against at-floor counts of 109/99/62 — so that chart takes the two-state split on the
categorical pair instead.

A banded chart drops its per-trace labels. Three of them ran across the densest part of the cloud
saying what the legend already said.

### A model drawn against the record it is a model of

`/method` had four tables and no chart, on the page whose subject is which figures here are models
and which are records. It now draws one against the other: mills `crates/millage` predicts from H.B.
920's reduction factors against mills the county auditor charged, with the line where they agree.

The finding is where the agreement is. **182 of the 273 districts at the twenty-mill floor land on
the line, against 6 of the 336 above it** — at the floor the factors have stopped operating and
there is nothing left to predict, and above it they are what sets the rate. The departures run one
way, 163 districts charging more than predicted by over half a mill against 11 charging less, which
is what factors that reduce existing levies on existing property and know nothing of a levy passed
since would produce. Each district's own tax page already said which its residual is consistent
with; this is the same statement about all 609 at once.

Two things about the drawing are load-bearing rather than cosmetic, and the first version got the
second one wrong. **The axes share a domain**, because a line through the corners of two different
ranges is not y = x. And **the plot area is squared**, because a shared domain on the ordinary
640×420 frame still draws the line at 33° — which reads as a trend the cloud is beating rather than
as the equality it is. An e2e test measures the rendered line's bounding box and fails if its aspect
leaves 45°.

### And a position is drawn against the population it is a position in

`distributionSpec` is the sixth form, and it replaced a flat bar with a pin in it. The bar had the
minimum at one end, the maximum at the other and nothing between, so the 60th percentile and the
95th were drawn identically — when Ohio's assessed valuation per pupil reaches **5.5× its median**,
those are a dense middle and open country. The same defect applied wherever a peer group was reduced
to its extremes: a county page named its richest and poorest district and drew neither the fifteen
between them nor how they were spread.

One form, four placements — the district position card, the county spread, a legislative seat's
schools, and a district's poverty fifth on the outcome route. **The form decides how to draw itself
rather than four call sites deciding four ways**, on two thresholds that are in the module and
nowhere else:

- **Members are drawn individually up to 150.** A county has six districts at the median and a
  poverty fifth has 122; both fit across 640px in five lanes and both are worth seeing. Ohio's 609
  do not — six hundred marks on a 46px strip is a rule — so above that the box carries the shape and
  only the districts past the fences are drawn. The first version had this backwards and rendered an
  *empty frame* for the poverty fifth: 122 districts, none beyond the fences, so "outliers only"
  meant no marks at all.
- **The box is drawn from 8.** Below it the quartiles of *n* numbers are just some of the numbers. A
  seat with three districts drew a box spanning nearly the full width with three dots in it. **39 of
  Ohio's 132 legislative seats and 60 of its 88 counties are under the floor**, so this is the
  common case, not the edge one — and each of them is better served by the dots alone.

The vertical spread on the dots is index-based, not random: this module is pure, and a jitter that
moved between builds would redraw one county two ways. Whiskers stop at the last value inside 1.5
IQR and anything past them is its own mark, which is what keeps Ohio's one $1.35M district from
being drawn as the end of a continuum it is nowhere near.

## Preview cards are the same idea, rasterized

A link to any page here unfurls with a card carrying that page's own figures: a district's aid per
pupil and how much of it is guarantee money, a county's internal wealth gap, a corpus node's
definition. [`src/lib/og/`](src/lib/og/) lays each one out with `satori` and rasterizes it with
`resvg`, and the endpoints under [`src/pages/og/`](src/pages/og/) emit **995 PNGs** — one per
district, county, legislative seat, ontology class, corpus node and catalog source, plus the
fourteen written ones in [`pages.ts`](src/lib/og/pages.ts).

A district's five routes share one card. What separates them in a feed is `og:title`, which carries
each page's own title, so five renderings per district would be 3,045 images for a difference no
reader would see.

**This is the one surface where a literal colour is correct**, which makes it the exact inverse of
the rule above. A chart is SVG in a document and defers its colours to the reader's theme; a card
is a PNG handed to Slack, with no cascade behind it and no way to re-render. So
[`palette.ts`](src/lib/og/palette.ts) writes the light-mode values out, and `tests/unit/og.spec.ts`
parses the `:root` block of `app.css` and fails if the two have drifted. A card must never be
routed through `renderToString`, which would reject it.

Two things to know before touching the renderer:

- **`loadSystemFonts: false` is load-bearing.** resvg builds a font database per instance, and by
  default that means scanning every font on the machine. Measured on this card: 121 ms with the
  default, 9 ms without — across ~1,000 cards, two minutes against nine seconds. It is free because
  satori embeds glyph outlines, so no `<text>` reaches resvg at all.
- **The card has one layout and takes no layout argument.** It is read at 400 pixels wide, in a
  feed, by someone who has not decided to look at it. The variable worth spending is which figure,
  not where it sits.

The layout also fixes what the canonical link had been claiming. `build.format` is `"file"`, so
`Astro.url.pathname` is the *output file* — the site shipped `<link rel="canonical">` pointing at
`/district/043786.html` while the sitemap beside it listed `/district/043786`. `canonicalPath` in
[`routes.ts`](src/lib/routes.ts) puts the served path back, both tags are built from it, and an
artefact test now holds the two files to the same set of addresses.

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

## Every section has an address, and now says so

A card carries an `id` that is also its `data-part`, and heads itself with a muted `#` linking to
that id. **121 cards and 248 prose headings**, 21,504 anchors across the built site, and one
definition of the markup in
[`src/lib/section.ts`](src/lib/section.ts) — the template literals in `src/lib/` call it, the
`.astro` templates render it through `SectionAnchor.astro` rather than restating it.

The addresses came first and went unused. 21 names in `SECTIONS`, all on the district routes, and
**two links in the repository ever named one** — because an address a reader cannot see is an
address nobody asks for. So the two halves shipped together: 80 cards that had no `id` got one, and
every heading got the link that makes its `id` visible. `SECTIONS` in
[`src/lib/routes.ts`](src/lib/routes.ts) is now grouped by route family and covers the whole site.

It is a bare `<a href="#…">` for the reason the basis switch is two radios: a third of the e2e suite
runs with JavaScript off, and a control that dies without script is worse than no control. It is
permanently visible rather than revealed on hover, which is the same objection the stylesheet
already makes to `title` — a hover-only affordance does not exist on a touch screen.

Three checks in `check-dist-links.ts` hold it, and each closes a hole that was already open:

- **Same-page fragments were checked by nothing.** The pattern collecting fragment links required a
  leading `/`, so `href="#base-cost"` — the most hand-written shape on the site — was invisible to
  the dangling-fragment check. 22,026 fragment links now resolve; 516 of them were being checked.
- **A duplicate `id` was invisible**, because a page's ids went into a `Set`. The check found one
  immediately: `#actuals` was on every `/district/*/finances` page **twice**, once per dollar basis,
  and a browser sent there scrolls to the nominal panel whether or not that is the one selected. The
  address moved up to the `.basis-scope` holding both panels, which is what it always meant.
- **Nothing stopped a new card shipping unaddressed**, which is how 80 of them accumulated. Every
  card in `main` must now carry an `id` this table lists, and a card that heads itself must carry
  the anchor pointing back at it — the two are written separately and this is what keeps them from
  drifting.

### The contents list is read off the page, not written beside it

A page of four sections or more lists them above the first one. The list is **derived from the
rendered body** in [`src/lib/contents.ts`](src/lib/contents.ts) — `Base.astro` renders its slot to a
string, reads the headings out of it, and inserts the list before the first section.

That is the whole design, and the reason for it is that a section on these routes is conditional.
`#denominators` is absent for 177 of 609 districts, `renderOutcomes` returns nothing for a feed with
no outcome block, a district with no five-year filing has no `#actuals` at all. **A declared list is
a claim about what renders rather than a record of it**, and it is wrong silently — the entry looks
like every other entry and lands the reader at the top of the document. Whatever is in the page is
in the list, and nothing else can be.

Every `h2`, plus the `h3`s the corpus prose grew — and not the `h3`s a card grew. A node's
description *is* the page and its headings are what its author divided the argument into; the
district dashboard's eight `h3`s are the six categoricals plus transportation and preschool, which
are the rows of one breakdown. `/wiki/doctrine/equity` lists fifteen sections, `/method` eight,
`/counties` none.

Two things fall out of deriving it. A section rendered once per dollar basis has two headings and
one address, so an entry is named by **what its headings agree on** — "…and hold — nominal" against
"…and hold — FY2020 dollars" gives "What districts actually received, spent, and hold". And every
entry is a same-page fragment, so `check:dist` already fails the build on one that names nothing:
33,926 fragment links now resolve, against 516 before any of this.

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

The site is served at **<https://ohio-education-funding.pages.dev>** from Cloudflare Pages,
project `ohio-education-funding`. Publishing is automatic: the `deploy` job in
[`.github/workflows/ci.yml`](../.github/workflows/ci.yml) runs on every push to `main` that clears
all three gates, and uploads the `dist/` the browser suite ran against rather than a rebuild of
it, so what is served is what was tested.

Publishing by hand is the escape hatch, and `mise run //:deploy` is the short form of it. The long
form, from the repository root:

```
pnpm --dir web test                       # both suites
pnpm --dir web build
pnpm dlx wrangler@4.125.0 pages deploy web/dist \
  --project-name ohio-education-funding --branch main
```

Three things about that third line are easy to get wrong and quiet when you do. `--dir` steers which
package pnpm runs, not where `dlx` resolves paths, so the directory argument is `web/dist` and not
`dist` — passed the latter from the root, wrangler goes looking at the repository root and exits
`ENOENT`. And `--branch main` is not decoration: on direct upload wrangler labels the deployment
with whatever branch is checked out, and only the production branch reaches the production URL.
Deploy from a `phase/*` branch without it and the upload succeeds, wrangler prints a URL, and the
live site does not change — you have published a preview. And the wrangler version is pinned, to
the same `4.125.0` the workflow's `wranglerVersion` names: `@latest` on a command that carries a
production credential runs whatever was published this morning, and an escape hatch that ran a
different wrangler than the automated deploy would eventually disagree with it about what a direct
upload means. Bump both together or neither.

**A feed change is now a rebuild.** `cargo run -p bundle` followed by the three commands above,
in that order. The old shortcut of regenerating `public/data/bundle.json` alone no longer updates
what a reader sees, because the figures are in the HTML.

`dist/` is about 201 MB across 4,562 files, well inside Cloudflare Pages' 20,000-file limit — the
file count is what to watch, and it has room. 21.5 MB of the weight is the 1,050 preview cards
under `og/`, which is the price of a shared link carrying the district's own figures. They are
indexed PNGs: a card is 128 colours and every pixel opaque, so `src/lib/og/indexed.ts` writes the
palette resvg's truecolour output was spending four bytes a pixel to avoid.

There is no custom domain, and this section used to say there was.
`schools.ohio.shawneesmart.systems` was named as the address for long enough to reach every
canonical link, the sitemap and a thousand preview cards, while never resolving at all — the
record was never created. That is why the address above is the Pages project's own subdomain: it
is the one that answers.

The plan for a custom domain is kept here rather than lost. DNS would be split, and knowing which
half is which saves an hour when something breaks. `shawneesmart.systems` is authoritative on
**Route 53** and stays there — it carries the mail records. The only thing Cloudflare would own is
one `CNAME` at `schools.ohio` pointing to `ohio-education-funding.pages.dev`, plus the certificate
it issues for that hostname. Cloudflare Pages custom domains work against external DNS this way on
the free plan; a Workers deployment would not, because a Workers custom domain requires the whole
zone to live on Cloudflare.

The order that setup happens in matters and is not recoverable in the moment: the hostname must be
registered on the Pages project **before** the `CNAME` exists in Route 53. Reversed, the record
resolves to a Pages edge that has never heard of the hostname and answers 522 until the association
catches up. And `astro.config.mjs`'s `site` moves in the same commit as the `CNAME`, never after
it — pointing the canonicals at a hostname that does not answer yet is the failure this section is
a record of.

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

`satori` and `@resvg/resvg-js` joined them for the preview cards. Neither is hand-rollable at any
sensible cost — one is a flexbox implementation, the other an SVG rasterizer — and both run only
during the build. `@fontsource/inter` is there because a card is rasterized on a build server,
which has no "system UI sans" to resolve; it brings the SIL licence with it, which committing a
`.ttf` would not.

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
src/lib/prose.ts      corpus markdown: link rewriting, claim badges, heading anchors
src/lib/section.ts    the section anchor: one definition, rendered by both halves of the site
src/lib/relationships.ts  district pairs and median traces; no fitted line lives here
src/lib/contents.ts   what is on this page, read off the rendered body at build
scripts/              schema emission and the corpus report; run by pnpm, not by the build
src/lib/plot/         chart specifications, and the two renderers that share them
src/lib/og/           preview cards: the palette, the layout, and the rasterizer
src/pages/og/         one endpoint per card family; 995 PNGs, none of them in the sitemap
src/lib/*.ts          the formula, the views, the formatting
src/scripts/          the client entries; small, and never load-bearing
public/data/          the feed, copied verbatim into dist/
public/favicon.svg    the site mark; the two icon routes rasterize this one file
public/_headers       cache and security headers, read by the host
tests/unit/           the formula, the projection, the link graph, the 404 matcher
tests/e2e/            the site in Chromium, a third of it with JavaScript disabled
```

## What is not built

The **regime view** — walking the numbers from the Foundation Program to the Fair School Funding
Plan — in the formula's own terms. That still needs a per-district historical series this
repository does not have.

What `/history` gives instead is the same span measured from outside: the Census Bureau's survey
of every comparable Ohio school system, FY2009 through FY2022, which spans the Bridge Formula and
the Fair School Funding Plan and says what happened to the revenue mix and the equity gap across
both. It is not the formula and it does not reconcile with it — a different population on a
different pupil count — and the page says so before it says anything else. The route exists
separately from the statewide view for that reason.

**Year-over-year funding.** The district view compares the FY2027 formula run at each of three
enrollment years, which isolates the enrollment channel exactly. It is not a comparison of
published FY2026 and FY2027 totals, and it says so on the card: the department publishes one
foundation calculator at a time and replaces rather than archives it, so no FY2026 per-district
payment figure exists in this repository. Getting one is a
[`dew-foundation`](../crates/connect/sources/dew-foundation.md) source problem, not a rendering one.

**The community school and scholarship deduction.** Not modelled; the 609 districts here are the
traditional districts in the state's own calculator.

## Bundle status

<!-- REGEN: edfund-connect bundle-status
Regenerated by: `edfund-connect index`
Fields: bundle contract version, feed list, last export timestamp, node counts per feed,
        deployment target, last deploy status.
-->
| Field | Value |
|---|---|
| Contract version | `39.0.0` |
| Districts in the feed | 609 |
| Reference checkpoints | 8 |
| Reference forecasts | 4 |
| Size | 6118 KB |
| Deployment target | Cloudflare Pages, static, with a CSP in `web/public/_headers` |

Regenerate with `cargo run --manifest-path crates/Cargo.toml -p bundle > web/public/data/bundle.json`. CI fails if the committed feed and a fresh one differ.
<!-- /REGEN -->
