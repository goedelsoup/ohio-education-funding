/**
 * Chart colour, as references into the stylesheet rather than as colour.
 *
 * # Why there is not a hex code in this file
 *
 * Almost every chart on this site is rendered to SVG at build time, and a build-time SVG cannot
 * re-render when the reader switches theme. Baking `#2a78d6` into a `fill` would leave every
 * chart in light-mode colours on a dark page. Handing Plot a `var(--…)` instead defers the choice
 * to the browser, which resolves it against the same custom properties the rest of the stylesheet
 * uses — so dark mode stays one CSS cascade and never a second render.
 *
 * `var()` in an SVG presentation attribute resolves and inherits exactly as it does in CSS, which
 * is what makes this work; `ensureThemeable` in `ssr.ts` fails the build if a literal colour ever
 * slips past, because the resulting bug is invisible to whoever writes it.
 *
 * The values behind these names are in `app.css` and are two series in slots validated for both
 * surfaces — `#2a78d6`/`#eb6834` light, `#3987e5`/`#d95926` dark. All six checks pass in both
 * modes: lightness band, chroma floor, CVD separation (ΔE 24.7 protan, worst case), normal-vision
 * separation, and contrast against the card surface. Dark is a selected step from the same ramps
 * rather than an inversion.
 *
 * Two series is the whole categorical palette, and there is no code anywhere here that generates
 * a third. A chart needing more is the wrong chart.
 */

/** The categorical pair, in fixed order. Never cycled, never generated. */
export const SERIES = {
  /** Formula aid. Reused as "gain" on the diverging surfaces, and only there. */
  formula: "var(--series-formula)",
  /** Guarantee. Reused as "loss". The contrasting half of the validated pair. */
  guarantee: "var(--series-guarantee)",
  /** The diverging midpoint, and any mark with no polarity. Never a hue. */
  neutral: "var(--neutral-mark)",
} as const;

/** Ink. Text wears a text token and never a series colour; a mark beside it carries identity. */
export const INK = {
  primary: "var(--text-primary)",
  secondary: "var(--text-secondary)",
  muted: "var(--text-muted)",
  /** Axes and rules, deliberately recessive. */
  rule: "var(--accent-rule)",
  /** The card the chart sits on. Used for the ring on overlapping marks. */
  surface: "var(--surface-1)",
} as const;
