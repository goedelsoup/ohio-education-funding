/**
 * The glyphs the bar draws, as path data.
 *
 * Inline rather than fetched, for the reason the footer's repository mark gives: the content
 * security policy admits no third-party origin, and a few short paths are cheaper than a request
 * would be even if it did. Drawn here on a 20-unit grid, stroked rather than filled, so each takes
 * `currentColor` and follows the ink of the link it sits in, in either theme.
 *
 * Only the record at the head of `Library` uses them — see `NavGroup.lead` in `nav.ts`.
 */

/** The 20-unit square each glyph is drawn on. */
export const ICON_VIEWBOX = "0 0 20 20";

/** Every glyph, as the `d` of each stroked path it is made of. */
export const ICONS = {
  /* Two books upright and a third leaning on them: the shelf, which is every class at once. */
  corpus: ["M2.5 3.5h3v13h-3z", "M7.5 3.5h3v13h-3z", "M12.4 4.6l2.9-.8 3.2 12.1-2.9.8z"],
  /* A page with its corner turned and two lines of text: the document a claim points into. */
  sources: ["M4.5 2.5h7l4 4v11h-11z", "M11.5 2.5v4h4", "M7.5 10.5h5", "M7.5 13.5h5"],
  /* One path that forks: a decision is the branch taken, recorded with the one not taken. */
  decisions: ["M10 17.5v-6.5", "M10 11L4.5 5.5", "M10 11l5.5-5.5", "M4.5 9.5v-4h4", "M15.5 9.5v-4h-4"],
} as const satisfies Record<string, readonly string[]>;

/** The name of a glyph in {@link ICONS}. */
export type NavIcon = keyof typeof ICONS;
