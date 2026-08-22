/**
 * Colour for the preview cards, as literal hex — the one place in this repository where that is
 * the correct thing to write.
 *
 * # Why this is the exact inverse of the chart rule
 *
 * `src/lib/plot/ssr.ts` fails the build if a chart SVG carries a literal colour, and the reason is
 * sound: a chart is rendered to SVG once at build time but *displayed* by a browser that may be in
 * either theme, so a baked `#2a78d6` leaves every chart in light-mode colours on a dark page. The
 * fix there is to hand Plot a `var(--…)` and let the cascade resolve it per reader.
 *
 * A preview card has no cascade to defer to. It is a PNG, rasterized at build time and handed to
 * Slack or to a crawler, and `var(--surface-1)` in it resolves to nothing at all. So the card must
 * carry literals, and it must never be routed through `renderToString` — which would reject it.
 *
 * # Why light, and only light
 *
 * The Open Graph protocol has no mechanism for offering two images and letting the client pick the
 * one matching its theme. One image is all there is, so it is the light one: it is what the site
 * serves by default, and a light card on a dark timeline reads as a document, while a dark card on
 * a light one reads as a hole.
 *
 * # Why the values are duplicated rather than parsed
 *
 * Reading `app.css` at build time would be a stylesheet parser in the render path for five colours.
 * Instead they are written here and `tests/unit/og.spec.ts` parses the `:root` block and fails if
 * these have drifted from it. A theme change then breaks a test — the alternative is cards that
 * quietly keep last month's colours, which nobody looks at often enough to notice.
 */

/** The light-mode `:root` values from `src/styles/app.css`, by their custom-property name. */
export const CARD = {
  /** `--surface-1`. The card face. */
  surface: "#fcfcfb",
  /** `--surface-2`. The unfilled half of the proportion bar. */
  surfaceMuted: "#eeeeea",
  /** `--text-primary`. The headline. */
  text: "#0b0b0b",
  /** `--text-secondary`. The line under the headline. */
  textSecondary: "#52514e",
  /** `--text-muted`. The eyebrow and the footer line. */
  textMuted: "#66655f",
  /** `--series-formula`. Formula aid, and the figure when it is not a guarantee figure. */
  formula: "#2a78d6",
  /** `--series-guarantee`. Guarantee aid. The contrasting half of the validated pair. */
  guarantee: "#eb6834",
  /** `--accent-rule`. The hairline under the eyebrow. */
  rule: "#c9c8c1",
} as const;

/**
 * The custom property each card colour is copied from.
 *
 * Exported for the drift test rather than for the renderer — it is what lets the test say
 * "`CARD.formula` no longer matches `--series-formula`" instead of "two hex codes differ".
 */
export const SOURCE_PROPERTY: Record<keyof typeof CARD, string> = {
  surface: "--surface-1",
  surfaceMuted: "--surface-2",
  text: "--text-primary",
  textSecondary: "--text-secondary",
  textMuted: "--text-muted",
  formula: "--series-formula",
  guarantee: "--series-guarantee",
  rule: "--accent-rule",
};
