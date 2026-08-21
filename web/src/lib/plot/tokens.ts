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

/**
 * The ordinal ramp: three steps of one hue, light to dark.
 *
 * Separate from {@link SERIES} because it answers a different question. The pair above is
 * *identity* — formula against guarantee, two things that are not versions of each other. This is
 * *order*: a district's band within a measure, where swapping two bands changes the meaning. An
 * ordered grouping drawn in categorical hues makes a reader look up what each one is; drawn in one
 * hue's steps, the order is in the colour.
 *
 * Three steps and not five, and the ordering is real even though the figures that used to sit here
 * were not reproducible. A scatter is an all-pairs form — any band can sit beside any other, so
 * every pair must separate, not merely adjacent ones — and five steps of one hue do not.
 *
 * **The measurements are in `web/tests/unit/palette.spec.ts` now, and they are not the ones this
 * comment used to state.** It claimed a normal-vision ΔE of 21.4 light and 21.6 dark against a
 * five-step failure at 10.9, sourced to a validator no longer in the repository. Nothing
 * reproduces 21.4: CIE76 gives 31.1, CIE94 23.0, CIEDE2000 17.9, OKLab 18.1. Measured in CIEDE2000
 * across normal vision and three dichromacies, this ramp's worst pair is **15.0 light and 17.1
 * dark**, and a five-step ramp built to escape the problem lands at 10.9 — the very number the old
 * comment used to reject five steps.
 *
 * So the conclusion held and the arithmetic behind it did not exist. It does now.
 *
 * A chart using this must carry a legend. The end steps sit near 2.2:1 against their own surface,
 * which is a contrast warning that obligates relief rather than one that can be waved off.
 */
export const ORDINAL = [
  "var(--ordinal-1)",
  "var(--ordinal-2)",
  "var(--ordinal-3)",
] as const;

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
