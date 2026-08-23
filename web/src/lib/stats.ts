/**
 * The summary statistics this layer computes for itself, defined once each.
 *
 * Both of these existed twice. `median` was written out in `county.ts` and again in
 * `relationships.ts` — same sort, same upper-middle element, same zero for an empty list, and no
 * reason for either to know about the other. The equal-count partition was written out in
 * `statewide.ts` (by valuation) and again in `outcomes.ts` (by poverty), identical down to the
 * comment explaining that the last group takes the remainder.
 *
 * Neither pair disagreed. That is the argument for merging them rather than against it: two
 * copies that agree today are a drift that has not happened yet, and this repository has already
 * paid for that once on the crate side, where a workspace carried two definitions of median and
 * published figures from both.
 *
 * The crates remain the authority for anything the formula depends on. Nothing here computes a
 * funding figure; these summarise figures the feed already carries.
 */

/**
 * The upper-middle value, or zero for an empty list.
 *
 * Upper-middle rather than the mean of the two middles, which is what both copies did and what
 * every caller's prose says — "the median district" names a district, so the statistic has to be
 * a value one of them actually has.
 */
export function median(values: number[]): number {
  const sorted = [...values].sort((a, b) => a - b);
  return sorted.length === 0 ? 0 : sorted[Math.floor(sorted.length / 2)]!;
}

/**
 * Five groups of equal count, ordered by `key` ascending.
 *
 * The last group takes the remainder, so integer division drops nobody: at n=609 the groups are
 * 121, 121, 121, 121, 125 rather than five of 121 with four districts silently gone. It also
 * means the last group is the largest by up to four, which matters for a median but not for the
 * share-of-group both callers take.
 *
 * Filtering is the caller's: `statewide.ts` wants districts with a valuation, `outcomes.ts` wants
 * those with both a poverty share and a performance index, and neither can be inferred from here.
 */
export function quintiles<T>(items: T[], key: (item: T) => number): T[][] {
  const sorted = [...items].sort((a, b) => key(a) - key(b));
  const size = Math.floor(sorted.length / 5);
  return Array.from({ length: 5 }, (_, i) =>
    sorted.slice(i * size, i === 4 ? sorted.length : (i + 1) * size),
  );
}
