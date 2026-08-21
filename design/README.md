# Design system

This repository is associated with a Claude Design project, **Ohio School Funding Design System**
(`62873473-0a8e-42b5-8c3d-3b44b299808f`). The project was authored from a brief describing this
site's needs, and it reads back against `web/` — its own README names `web/src/styles/app.css`,
`web/src/lib/plot/tokens.ts`, `web/src/lib/og/` and `web/src/lib/prose.ts` as its ground truth.

It is not a vendored dependency. Nothing here is generated, and the design project is not built
from this directory. The two are separately maintained and have to agree anyway, which is the same
arrangement the preview-card palette has always had with the stylesheet — and the same reason that
agreement is asserted by a test rather than assumed.

## What has landed

**The token layer, and only the token layer.** `web/src/styles/tokens/` holds `colors.css`,
`typography.css` and `space.css`; `app.css` imports the three and no longer declares tokens of its
own.

The colour tokens landed **byte-identical in both modes** — fourteen properties, nothing changed,
nothing dropped, verified by diffing the old `:root` blocks against the new files before wiring
anything. What the system added on top is nine tokens the site did not have: a five-step ordinal
ramp, three claim colours, and semantic aliases.

Type and space tokens exist now and **nothing consumes them yet**. The rules in `app.css` still
carry their own literals. Moving them onto the scale is the next step and is deliberately not this
one, because that is a change with a visual diff and this was not.

Two mechanisms changed and neither is visible:

- The dark override was `:root:where(:not([data-theme="light"]))` inside the media query. It is now
  a bare `:root` in the query with `[data-theme="light"]` and `[data-theme="dark"]` re-stating each
  palette at higher specificity. An explicit choice still beats the OS setting in both directions;
  the theme tokens are now discoverable as theme tokens rather than sitting behind what reads to
  tooling as a component selector.
- The preview-card drift test now parses `tokens/colors.css`. It parsed `app.css` and failed the
  moment the tokens moved, which is the check working. It also now asserts that it found a `:root`
  block at all, so a future move fails loudly rather than silently comparing against nothing.

## One defect found on the way in

`tokens/colors.css` and `tokens/typography.css` both declared `--text-body` on `:root`, as a colour
and as a font size. `patterns/prose.css` consumes it as a size, and the entry point imports colour
first, so the size already won — the colour alias was dead on arrival and the collision was masked
by import order. Reordering those two lines would have set every `.prose-body` font-size to a
colour, which computes to invalid and falls back to inherited.

Renamed here to `--text-ink`. The rename cannot break anything that works today, because nothing
that works today reads the shadowed name.

## What has not landed, and what it costs

The eight pattern sheets are the rest of the system and each carries a real change:

| Sheet | Why it is not a copy-in |
| --- | --- |
| `claims.css` | Removes the box from `.claim` — no border, no background, no radius — and carries status on a coloured rule whose *style* is the second channel. It also drops the fourth inline variant, which this corpus emits **76 times**. |
| `figures.css` | Introduces `.fig` as a compound element: value, year, basis as three children, so an unlabelled number is a missing child a build check can see. The site annotates *cards* with a year today, not figures. |
| `prose.css` | Four fields, three visual channels — size, ground, state. Close to what ships; the `.findings` inset and the `<details>` withdrawal are new treatments of fields that already exist. |
| `base.css`, `cards.css`, `data.css`, `nav.css`, `controls.css` | Not yet read. |

Landing them means rewriting rules that 293 unit tests and 238 end-to-end tests assert against, so
each is its own change with its own diff.

## All eight pattern sheets are read

`base.css`, `cards.css`, `data.css`, `nav.css` and `controls.css` were the five outstanding. Read on
2026-08-21; the delta is recorded upstream in the project's own `PATTERNS-REVIEW.md`. Two results
matter here.

**The `--text-body` collision is resolved, and the fix upstream is safe.** The property appears
exactly once across all eight sheets — in `prose.css`, as a font size. Nothing anywhere reads it as
a colour, so the colour alias is unreachable under the import order `styles.css` declares and
renaming it cannot break anything. This repository already renamed its copy to `--text-ink`; the
same one-line change upstream removes a `:root` property whose value depends on which of two files
loaded last.

**`data.css` declares five `--ord5-*` legend swatches.** Consistent while the tokens exist, and this
repository has deleted them. If the design project follows the measurement, those five rules go with
the tokens — a legend entry for a ramp nothing may draw is a legend for a chart that cannot exist.
Left as the project's decision.

The other five sheets are very close to what ships. The differences are almost entirely literals
that should name tokens, which is the move `prose.css` already made. Two real additions worth
having: `.variants`, which reveals one of N build-time chart variants by attribute selector — the
basis switch's trick applied to charts, so a sort change swaps a pre-rendered strip instead of
loading a plotting library — and `tbody + tbody` group structure for long tables, which is also what
carries a definitional break.

## `figures.css`, and the number that decides the rest of it

The patterns are landed. `.fig` is **available and not yet adopted**, and the number is why.

The site formats figures through `format.ts` from **750 call sites across 28 files**. The design's
rule 1 — no numeric or currency literal outside a `.fig` — means wrapping every one of them in a
three-child compound. That is a change worth deciding on with the number in hand rather than
starting and discovering it, so the CSS is in place, the shape is documented at the pattern, and the
adoption is a separate decision.

Where the gap is real: the card-level year chip answers "which year" well for a card, and does not
reach body prose, a chart label, or a table cell whose column head is silent. That is the case for
`.fig` and it is a good one. It is not a case for 750 mechanical edits in one pass.

**`.series-break` is adopted, because there the gap was concrete.** The meal-program denominator
changes at FY2010 — `AdmCount` through FY2009, `CECount` after — and the share steps up across it.
The chart has drawn that as two separate series since it was built. The table beside it ran straight
through the same break, carrying the distinction only in a per-row "Counted on" cell, which a reader
meets after the eye has already gone down the column. It now closes one row group and opens another
with a labelled rule between them: structure a screen reader announces, not a border only a sighted
reader sees.

An end-to-end assertion counting `tbody tr` failed at 18 against an expected 17 when that landed,
which is the check noticing that the table's shape changed — and the shape is the point. It counts
data rows now, and a second assertion holds the break itself.

## Two decisions the design project asked for, and neither is a CSS question

**1. The `unentered` migration.** The system's `.claim` has exactly three status classes and states
that a fourth is an error. This corpus marks 76 fields `[unentered]`, and `web/src/lib/prose.ts`
badges them as a fourth inline variant. The design argues the fourth state is not a weaker grade of
support but the absence of a claim to grade, so it belongs in the position the content would
occupy — a block, naming the field — rather than inline in a sequence it does not belong to.

That argument is sound and the corpus already agrees with it in prose: `.yidam/corpus/README.md`
says the mark "sits on a different axis" from the other three. Adopting it is a parser change plus a
content migration, not a stylesheet edit. **Unresolved.**

**2. The five-step ordinal ramp — answered, and the answer is no.**

The design asked for six checks across all ten pairs, both modes, under protan, deutan and tritan
simulation, and said it would rather ship four measured steps than five designed ones. That has now
run. `web/src/lib/plot/palette.ts` carries the arithmetic — sRGB to CIELAB, CIEDE2000, the Machado
dichromacy matrices at severity 1.0, WCAG luminance — and `web/tests/unit/palette.spec.ts` runs it
against the tokens themselves rather than against copied values.

| CIEDE2000, worst over normal + three dichromacies | light | dark |
| --- | --: | --: |
| 3-step ramp | 15.0 | 17.1 |
| 5-step ramp | **10.9** | **10.7** |

**10.9 is the number this repository already cited as its reason for refusing five steps** — "two
bands a reader with full colour vision cannot tell apart". The second channel the ramp added, a
30-degree hue drift alongside the lightness march, did not buy the separation it was introduced to
buy. The worst pair is 1–2 in both modes and it is *adjacent*, which is the pair a reader compares
most in the sorted table and choropleth the ramp was licensed for. Its lightest step also sits at
1.54:1 against its own surface, below the 2.2:1 the three-step ramp's end already treats as a
warning that obligates a legend and a table.

No four-step subset rescues it: dropping any single step reaches 11.9 light / 12.5 dark at best,
still short of 15.0 / 17.1. **Re-spacing within these five values is not enough — a five-step ramp
for this site has to be reconstructed rather than adjusted.**

**`--ord5-*` has been deleted from the tokens.** A ramp that fails its own validation and stays in a
palette file looks sanctioned; the search script carries that history better than dead tokens do.
`palette.spec.ts` now asserts that the ordinal vocabulary is exactly one ramp of three steps, and
that no declaration or `var()` reference to a five-step ramp exists anywhere in `src/` or `tests/`.

The design project still declares `--ord5-1..5` on its side. That is deliberate for now — the
decision to remove them there belongs with whoever picks one of the three options, and the file
already carries the measurement.

### The reconstruction was searched, and it does not reach five

`web/scripts/ramp-search.ts` optimises a ramp against the committed check rather than constructing
one and testing it afterwards. Monotone lightness with a real gap, hue inside one arc, chroma
bounded and moving smoothly, and the step nearest its own ground at 2.2:1 or better — which is what
an ordinal ramp is, written as constraints.

|  | steps | worst pair | bar | verdict |
| --- | --: | --: | --: | --- |
| light | 4 | 18.9 | 15.0 | clears |
| light | 5 | 15.1 | 15.0 | clears, barely |
| dark | 4 | 17.2 | 17.1 | clears by 0.1 |
| dark | 5 | **13.2** | 17.1 | **fails** |

**The dark surface is what binds.** Its contrast floor pushes the dark end of the ramp up in
lightness, leaving an L\* span of about 55 against light mode's 68, and tritanopia takes most of what
a blue ramp has left. Five steps do not fit in what remains.

Four fit in both modes, but dark clears by 0.1 — which is not a margin — and both winning ramps
drift off the site's blue into violet and pink. A ramp that clears the bar by abandoning the palette
has traded one problem for another.

**So the recommendation is option 1: three steps.** They are measured, licensed for every form
including all-pairs, and already shipping. A quintile gets position and a direct label, which is
what the site does today. The search script stays so the question stays answerable — if the
surfaces change or a wider hue budget is granted, run it rather than deciding again.

Two methodological corrections are recorded in the script's own header, because both were mine and
both would have produced a confident wrong answer. A first pass maximised separation over free Lab
points and returned a categorical palette, having been asked for separation and not for order. A
second added ordering but required every step to clear the contrast floor, which only the step
nearest its ground has to, and returned infeasible everywhere.

### And the three-step ramp's own figures did not reproduce either

The claim was ΔE 21.4 light / 21.6 dark. No standard metric produces it from the committed values:

| | light | dark |
| --- | --: | --: |
| claimed | 21.4 | 21.6 |
| CIE76 | 31.1 | 28.1 |
| CIE94 | 23.0 | 25.5 |
| CIEDE2000 | 17.9 | 19.0 |
| OKLab ×100 | 18.1 | 21.6 |

The contrast figure in the same comment — end steps at 2.2:1 — reproduces *exactly*, in both modes,
which is what establishes that the colour values and the sRGB chain are both right. The
disagreement is about the metric, not the colours.

**The decision those figures supported still stands.** Three steps do separate better than five
under every metric tested; that ordering never depended on the exact number. What did not exist was
the arithmetic. It does now, and the comments that stated the old figures say so rather than having
been quietly rewritten.

A third item — font binaries — is settled rather than open. `--font-sans` and `--font-mono` name IBM
Plex first and fall back to the platform stacks, which is what the site ships today at zero font
bytes. Tooling will report a missing `@font-face` for both families; that report is addressed to
whoever owns the files and is not a defect to resolve by substituting a different family.
