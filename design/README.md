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

## Two decisions the design project asked for, and neither is a CSS question

**1. The `unentered` migration.** The system's `.claim` has exactly three status classes and states
that a fourth is an error. This corpus marks 76 fields `[unentered]`, and `web/src/lib/prose.ts`
badges them as a fourth inline variant. The design argues the fourth state is not a weaker grade of
support but the absence of a claim to grade, so it belongs in the position the content would
occupy — a block, naming the field — rather than inline in a sequence it does not belong to.

That argument is sound and the corpus already agrees with it in prose: `.yidam/corpus/README.md`
says the mark "sits on a different axis" from the other three. Adopting it is a parser change plus a
content migration, not a stylesheet edit. **Unresolved.**

**2. The five-step ordinal ramp is designed, not measured.** The three-step ramp's separation figures
came from this repository's own validation — ΔE 21.4 light, 21.6 dark, 20.8 under the worst CVD
simulation. The five-step ramp's are the design's, and it says so plainly and asks for the same six
checks across all ten pairs in both modes under protan, deutan and tritan simulation before it
ships. Until that runs, `--ord5-*` is present in the tokens and used by nothing. **Unresolved, and
runnable here.**

A third item — font binaries — is settled rather than open. `--font-sans` and `--font-mono` name IBM
Plex first and fall back to the platform stacks, which is what the site ships today at zero font
bytes. Tooling will report a missing `@font-face` for both families; that report is addressed to
whoever owns the files and is not a defect to resolve by substituting a different family.
