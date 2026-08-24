/**
 * Saying what changed, on the three routes that rewrite themselves.
 *
 * # What was wrong
 *
 * `/scenario`, `/compare` and the 609 `/district/*\/scenario` routes replace their entire result
 * block when a lever moves. A reader who can see it watches the figures change; a reader who
 * cannot hears nothing at all, and the figures they came for have quietly become different
 * figures. The site had exactly two live regions before this — `#f-count` on `/districts`, and it
 * is correct — which is why their absence here reads as an oversight rather than a position.
 *
 * # Why the container is not the live region
 *
 * The obvious change is `aria-live="polite"` on `#scenario-out`. That announces the *whole*
 * rewritten block on every tick: four tiles, a distribution card, a ranked table of the
 * most-affected districts. Several hundred words, read out from the top, each time a slider moves
 * by one step. A live region has to be the size of the thing that is worth interrupting someone
 * to say, and that is a sentence.
 *
 * So the region is separate and small, and what goes in it is read back off what was rendered —
 * the same argument `contents.ts` makes about the table of contents. A summary composed
 * independently would be a second description of the result, free to drift from the first.
 *
 * Browser-side only, and it imports nothing: `scenario.ts` and `compare.ts` both run in the page.
 */

/**
 * A function that writes to the status line once the reader has stopped changing things.
 *
 * The debounce is not a nicety. A range input fires `input` on every step of a drag — pulling the
 * base-cost slider across its range is fifty events — and a polite region written fifty times in
 * two seconds is either fifty interruptions or, on a screen reader that coalesces them, a sentence
 * about a lever position the reader has already left. Waiting for the movement to stop says the
 * one thing that is true when they let go.
 */
export function saying(region: HTMLElement, wait = 500): (text: string) => void {
  let pending = 0;
  return (text: string) => {
    clearTimeout(pending);
    pending = window.setTimeout(() => {
      region.textContent = text;
    }, wait);
  };
}

/**
 * What a block of result tiles says, as one sentence.
 *
 * A tile is a `.k` naming a quantity and a `.v` holding it — "Total state aid, FY2032" and
 * "$6.83B – $7.21B" — which is already the shortest true statement of the result, because it is
 * what the page decided was worth showing largest. Reading it back rather than recomposing it is
 * what keeps the spoken summary and the printed one the same claim.
 *
 * `.n`, the note under each value, is deliberately left out. It qualifies the figure for someone
 * reading at leisure and it is a second sentence per tile, which is the length that makes a live
 * region something to switch off.
 */
export function tileSummary(root: ParentNode): string {
  return [...root.querySelectorAll(".tile")]
    .map((tile) => {
      const key = tile.querySelector(".k")?.textContent?.replace(/\s+/g, " ").trim() ?? "";
      const value = tile.querySelector(".v")?.textContent?.replace(/\s+/g, " ").trim() ?? "";
      return key && value ? `${key}: ${value}` : "";
    })
    .filter((part) => part !== "")
    .join(". ");
}
