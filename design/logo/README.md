# The logo

**The meridian mark is the site's icon.** It lives at `web/public/favicon.svg`, emitted from
`outline.ts` by `node design/logo/outline.ts` and held there by `mise run //:generated`.

Two concepts were drawn, both of them the state of Ohio with the silhouette left intact, and both
derived from the same thirty-six coordinates rather than drawn by hand. The calculator stays here
because it is the argument the chosen mark was chosen against, and because the measurement that
separated them is worth keeping.

| | |
| --- | --- |
| ![the meridian mark](../../web/public/favicon.svg) | **The meridian — adopted.** The Indiana border is the one perfectly straight edge the state has — a surveyed meridian from the Michigan line to the Ohio River — and the pencil occupies exactly that strip. The education motif and the geography are the same line. |
| ![the calculator](calculator.svg) | **The calculator.** The Fair School Funding Plan *is* a workbook: `fy26-calculator.xlsx` and `fy27-calculator.xlsx` are in this repository's corpus, and every figure the site publishes is something that workbook computes. The state carries a display over a keypad. |

Neither adds a shape outside the boundary. The pencil is a colour change inside the meridian strip,
with only its point leaving the outline below the Indiana–Kentucky–Ohio tripoint; the calculator
face sits wholly inside.

## Making the pencil an object rather than a stripe

Flush against the state and drawn as one flat colour, an orange rectangle with a point on it is
read as a band of colour in the silhouette before it is read as a pencil. Three changes fix that
without touching the concept, and none of them adds a hue:

- **A keyline.** A 0.9-unit hairline of the ground between the barrel and the state, so the pencil
  is lying *on* the map rather than being a stripe of it. It is drawn before the barrel, so the
  barrel's own edge stays clean and the line reads as ground showing through rather than as a
  stroke applied to the pencil.
- **A facet.** The barrel's highlight, a third of the way across, which is where a hexagonal
  pencil's facet edge falls. At a third rather than nearer the keyline: side by side, the two
  hairlines read as one narrow orange sliver that looks like a drawing error.
- **A longer point.** The graphite now takes 72% of the cone rather than 55%, so the point reads as
  sharpened rather than as a dark cap.

Three other treatments were drawn and dropped. A ferrule — two ink bands below the top — and an
eraser cap both give the mark **two dark ends**, and at 32px a reader cannot tell which end writes.
A second facet reads as a barcode. And the wood collar that would make the point two-tone has
nowhere to go: the cone sits on the ground rather than on the state, so a collar in the surface
colour is invisible exactly where it is needed.

## Where the calculator's face sits, and why it is not the obvious rectangle

The state's interior is not a rectangle, so the face has to be placed against the actual shape. Two
candidates, and the larger one is wrong:

| | x | y | w | h |
| --- | --: | --: | --: | --: |
| Largest rectangle that fits anywhere | 8.20 | 14.30 | 35.50 | 30.60 |
| Largest at a calculator's proportion, centred on the area centroid | 17.07 | 14.00 | 26.38 | 31.03 |

The first hugs the western meridian, because that is where the state's one straight edge is. A face
placed there is bigger and reads as slipped, since the eye centres on the middle of the shape and
not on the middle of the largest hole in it. The second is what `plate()` computes: the polygon's
area centroid fixes the horizontal position, the proportion fixes the aspect, and a binary search on
width finds the rest. Feasibility is monotone in width — the height follows from the aspect — which
is what makes that search valid rather than merely quick.

## The test that decided the first concept, and what it says about the second

`web/public/favicon.svg` states the constraint every mark on this site answers to: anything with
more in it than a couple of shapes "reads as a smudge in a browser tab, and that is the size which
decides whether a mark is worth having". Each candidate was rasterized at 180, 48, 32 and **16**
pixels and judged at the bottom of that ladder.

The two concepts fail differently, and the difference is measurable rather than a matter of taste.
Both marks have features that fall below a device pixel; what separates them is what is left when
those features go.

| Rendered at | Keypad's separator gap | The pencil's keyline |
| --- | --: | --: |
| 180px | 7.03px | 2.53px |
| 48px | 1.88px | 0.68px |
| 32px | 1.25px | 0.45px |
| **16px** | **0.62px** | **0.23px** |

The pencil's keyline and facet are *thinner* than the keypad's gaps, and it does not matter. A
sub-pixel line still renders as partial coverage — a lighter blend along one edge — and a blend is
all either of them has to be, because each is only saying "there is an edge here". The keypad's gaps
have a harder job: they have to separate six shapes into six readable objects, and partial coverage
cannot do that. At 16px the six keys stop being six keys and become one pale block.

What each degrades **to** is the rest of it. Lose the keyline and the facet and the mark is a pencil
flush against the state, which is a pencil. Lose the graphite and the point is solid orange, which
is still a pencil. Lose the keypad's gaps and there is no keypad.

**So: the calculator is the better mark from 32px up, and the meridian mark is the one that also
works at 16.** Which matters depends on where the mark is actually painted. `favicon.svg` is the
only surface below 32 — the header, the preview card, `icon-32.png` and the 180px apple-touch icon
are all at or above it, and a browser tab on a 2× display paints its 16 CSS pixels at 32 device
pixels, where the keypad's gap is 1.25px and the calculator holds.

## Concepts considered and dropped

| Concept | Why not |
| --- | --- |
| A mortarboard sitting on the state | The cap covers the Lake Erie shore, and the shore is what identifies Ohio. Below 48px the mark is a hat on a blob. |
| The state as the board, with a tassel | The tassel is a one-pixel-wide line at 16px, so it reads as a scratch on the icon rather than as a cord. |
| A mortarboard knocked out of the state | Two solid tones hold at any size, but the knockout at cap proportions reads as a face. |
| Notebook rules across the state | Horizontal bands across a silhouette read as a flag, and they cut the outline into pieces. |
| The state seated in an open book | Legible, and the runner-up among the first five. It stacks two objects, so the state gets small and at 16px the book is a bar. |
| An equals sign knocked out of the state | Durable and continuous with the two-bar icon, but "equals" says arithmetic rather than calculator, and two bars in a silhouette also read as a menu glyph. |
| The calculator body as the whole icon, state on the display | Charming, and it holds down to 32px — but the silhouette becomes a rounded square and the state becomes a detail inside it, which is the brief inverted. |
| The state on an adding machine's tape | The outer shape stops being Ohio, and the orange field takes over the mark from the blue. |

## Colour

Three values, all of them the site's own: the categorical pair for the state and the accent, and the
primary ink for the graphite. No new hue, which is the closed-set rule in
`web/src/styles/tokens/colors.css`.

They are written into the SVGs as **literals**, for the reason `favicon.svg` and
`web/src/lib/og/palette.ts` both give: a file served straight to a browser tab has no stylesheet,
and neither does the resvg pass that rasterizes it for the PNG icon routes.

Dark is a selected face rather than an inversion, carried in a `prefers-color-scheme` block that
names only the classes its own drawing uses. The two slots that have to move for a reason other than
taste are the graphite — near-black is invisible on the dark surface, so it takes the primary ink
there, which is white — and a calculator key, which is a hole and therefore takes whatever the
ground is.

resvg does not evaluate media queries, so anything rasterizing these files gets the light face. That
is the same trade `favicon.svg` records and it is correct for the same reason: an apple-touch icon
sits on a home screen whose background the site does not control.

## The files

| File | What it is |
| --- | --- |
| `outline.ts` | The source. Thirty-six named coordinates, both concepts, and the emitter. |
| `../../web/public/favicon.svg` | **The site icon**, on its rounded ground. Emitted here; not a copy. |
| `mark-bare.svg` | The same drawing with no ground, for a header or a lockup. |
| `lockup.svg` | Horizontal lockup, mark and wordmark. |
| `calculator.svg` | The calculator on its rounded ground. The concept not chosen. |
| `calculator-bare.svg` | The calculator with no ground. |

The five SVGs are **generated**. Edit `outline.ts`, never the SVGs:

```
node design/logo/outline.ts          # write them
node design/logo/outline.ts --check  # fail if the committed files disagree
```

The outline is a derivation and not a drawing on purpose. A hand-drawn path is a shape nobody can
check; this one can be re-derived, and a coordinate that is wrong is wrong at a place with a name.
The simplification is in the source too, and says what it dropped — Marblehead and Sandusky Bay are
one shoulder here, because at 16 pixels the peninsula is a stray pixel and at 180 it is a wobble
that reads as an error in the drawing.

## Two things to know

**The wordmark is set in the body stack, not the display serif.** That is the site's own rule rather
than a preference: `tokens/typography.css` reserves the serif for headings and the lead and excludes
anything "drawn inside an `<svg>`", because a platform stack is a promise an SVG cannot keep. The
preview card gets a serif headline because satori turns it into outlines first; `lockup.svg` has no
such step, so it renders in whatever the reading machine has. If the lockup ever needs the serif, it
needs an outlining step first, not a different font stack.

**The icon is emitted, not copied.** `web/public/favicon.svg` is the single source for the icon at
every size — `src/lib/og/mark.ts` reads it, and `icon-32.png` and `apple-touch-icon.png` are
rasterized from it — so it is the one output of this script that something else depends on by path.
It is written by `outline.ts` rather than exported by hand, and `mise run //:generated` re-emits it
and fails on a mismatch. That is the same argument the file's own predecessor made about hand-made
PNGs: the way an agreement between two copies breaks is invisible to whoever broke it.

Changing the mark is therefore an edit to `outline.ts` followed by `node design/logo/outline.ts` —
never an edit to the SVG, which the gate will reject.
