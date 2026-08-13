/**
 * What a preview card says, and how it is laid out.
 *
 * # The shape, and why it is fixed
 *
 * Every card on this site is the same five slots: an eyebrow naming the site, a rule, a headline
 * naming the subject, one figure with a note under it, an optional proportion bar, and a footer of
 * identifiers. Nothing here takes a layout argument.
 *
 * That is deliberate. A preview card is read at about 400 pixels wide, in a feed, for under a
 * second, next to things competing for the same attention — and the reader has not decided to look
 * at it yet. The variable worth spending is *which figure*, not where it sits. A generator that
 * could produce twelve layouts would produce twelve layouts, and the set would stop being
 * recognisable as one site.
 *
 * # Why the tree is object literals and not JSX
 *
 * satori accepts React elements, and it accepts the plain `{ type, props }` objects React elements
 * are. Taking the second means this is a `.ts` file that type-checks under the project's existing
 * config; taking the first would mean adding a JSX runtime and a React dependency to an Astro
 * project that has neither, in order to draw a rectangle with five text runs in it.
 */

import { CARD } from "./palette.ts";

/** Which series the figure and the bar belong to. Two, as everywhere else on this site. */
export type Tone = "formula" | "guarantee";

/** The content of one card. */
export interface Card {
  /** Above the rule. The site, or the section within it: "Wiki", "Cuyahoga County". */
  eyebrow: string;
  /** The subject. A district name, a county name, a corpus node's label. */
  headline: string;
  /** The one number worth carrying into a feed. Omitted when there isn't one. */
  figure?: string;
  /**
   * Which series the figure belongs to. Defaults to formula.
   *
   * Its own field rather than inherited from {@link bar}, which is what it used to be and was
   * wrong: on the statewide card the figure is *all* state aid and the bar is the guarantee's share
   * of it, so taking the bar's colour painted the total in the guarantee's orange. A colour on this
   * site names a series, and the largest thing on the card is the last place to let one drift.
   */
  figureTone?: Tone;
  /** What the figure is, in a few words. Only read if the figure earned the glance. */
  figureNote?: string;
  /** A proportion, 0–1, drawn as a bar with a label beside it. */
  bar?: { share: number; label: string; tone: Tone };
  /** The footer: identifiers, and the fiscal year the figures are from. */
  meta?: string;
}

/**
 * 1200×630 — the size every consumer of these has settled on.
 *
 * Facebook and LinkedIn want 1.91:1, X crops `summary_large_image` to 2:1, and Slack scales to fit.
 * 1200×630 is the intersection: it satisfies the ratio, clears X's 300px minimum by a wide margin,
 * and stays under every platform's 5 MB ceiling by two orders of magnitude at these colour counts.
 */
export const WIDTH = 1200;
export const HEIGHT = 630;

/**
 * Cut a string to a length a card can hold, on a word boundary.
 *
 * satori wraps, so nothing overflows horizontally — but a district name that wraps to three lines
 * pushes the figure off the bottom edge, and satori clips rather than complaining. Truncating is
 * the visible failure and is therefore the better one: an ellipsis reads as "there is more of this
 * name", where a silently cropped card reads as a broken renderer.
 */
export function clamp(text: string, limit: number): string {
  if (text.length <= limit) return text;
  const cut = text.slice(0, limit - 1);
  const space = cut.lastIndexOf(" ");
  return `${(space > limit * 0.6 ? cut.slice(0, space) : cut).trimEnd()}…`;
}

/**
 * How big the headline is set, by how much of it there is.
 *
 * Three steps rather than a continuous fit. A fitted headline makes every card a slightly different
 * document; three steps make "Lima City" and "Cuyahoga Heights Local" look like the same card with
 * different words in it, which is what a set of a thousand needs.
 */
function headlineSize(text: string): number {
  if (text.length <= 22) return 82;
  if (text.length <= 38) return 66;
  return 52;
}

/** A satori element. Structurally a React element, without needing React to say so. */
interface Element {
  type: string;
  props: Record<string, unknown>;
}

const row = (children: Element[], style: Record<string, unknown> = {}): Element => ({
  type: "div",
  props: { style: { display: "flex", alignItems: "center", ...style }, children },
});

const text = (content: string, style: Record<string, unknown>): Element => ({
  type: "div",
  props: { style: { display: "flex", ...style }, children: content },
});

/**
 * The card, as a tree satori can lay out.
 *
 * Read top to bottom: it is the picture in the same order the eye takes it.
 */
export function render(card: Card): Element {
  const children: Element[] = [
    // The eyebrow and its rule. Present on every card, so a reader who has seen one knows what
    // the next one is before reading the headline.
    text(clamp(card.eyebrow, 64), {
      fontSize: 26,
      fontWeight: 400,
      letterSpacing: "0.04em",
      textTransform: "uppercase",
      color: CARD.textMuted,
    }),
    {
      type: "div",
      props: { style: { display: "flex", height: 2, background: CARD.rule, marginTop: 18 } },
    },
    // The subject, and the only thing on the card set in the heavier weight.
    text(clamp(card.headline, 58), {
      fontSize: headlineSize(card.headline),
      fontWeight: 600,
      color: CARD.text,
      marginTop: 44,
      lineHeight: 1.1,
    }),
  ];

  if (card.figure) {
    children.push(
      text(card.figure, {
        fontSize: 54,
        fontWeight: 600,
        color: card.figureTone === "guarantee" ? CARD.guarantee : CARD.formula,
        marginTop: 26,
      }),
    );
  }
  if (card.figureNote) {
    /*
     * How much of the note there is room for depends on what is above it.
     *
     * A card with a figure has spent sixty pixels and a margin on it, and its note is a caption:
     * two lines. A corpus node has no figure at all — its "figure" is a definition — and the note
     * is the card's whole content, so it gets four. One limit for both wasted half the face on
     * every one of the 284 wiki cards and cut their definitions mid-clause.
     */
    children.push(
      text(clamp(card.figureNote, card.figure ? 110 : 190), {
        fontSize: 28,
        color: CARD.textSecondary,
        marginTop: 10,
        lineHeight: 1.35,
      }),
    );
  }

  // Everything above is flush to the top; everything below is flush to the bottom. `marginTop:
  // auto` on the first of the trailing elements is what splits them, so a card with no figure
  // does not leave its footer floating in the middle of the face.
  const trailing: Element[] = [];
  if (card.bar) {
    const share = Math.max(0, Math.min(1, card.bar.share));
    trailing.push(
      row(
        [
          {
            type: "div",
            props: {
              style: {
                display: "flex",
                width: 560,
                height: 20,
                background: CARD.surfaceMuted,
                borderRadius: 10,
                overflow: "hidden",
              },
              children: [
                {
                  type: "div",
                  props: {
                    style: {
                      display: "flex",
                      // Percent rather than pixels: the two agree here, and a percentage is the
                      // quantity the bar is *about*, so a changed bar width cannot silently rescale
                      // what it claims.
                      width: `${(share * 100).toFixed(2)}%`,
                      height: "100%",
                      background: card.bar.tone === "guarantee" ? CARD.guarantee : CARD.formula,
                    },
                  },
                },
              ],
            },
          },
          text(clamp(card.bar.label, 44), {
            fontSize: 26,
            color: CARD.textSecondary,
            marginLeft: 24,
          }),
        ],
        { marginTop: "auto" },
      ),
    );
  }
  if (card.meta) {
    trailing.push(
      text(clamp(card.meta, 88), {
        fontSize: 24,
        color: CARD.textMuted,
        marginTop: card.bar ? 22 : "auto",
      }),
    );
  }

  return {
    type: "div",
    props: {
      style: {
        display: "flex",
        flexDirection: "column",
        width: "100%",
        height: "100%",
        background: CARD.surface,
        color: CARD.text,
        fontFamily: "Inter",
        padding: "58px 64px",
      },
      children: [...children, ...trailing],
    },
  };
}
