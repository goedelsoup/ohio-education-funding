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
 * The median, by linear interpolation on rank — R's type 7, and `crates/dispersion::median`.
 *
 * # Why this changed, and what it cost
 *
 * It used to take the **upper of the two middle observations**, with a reason written beside it:
 * *"the median district" names a district, so the statistic has to be a value one of them actually
 * has.* That reason is a real one, and it was wrong about which problem it was solving.
 *
 * Merging the two hand-rolled copies into one function fixed a drift *inside* this layer while
 * leaving the larger one untouched. Eight `median_*` fields in `bundle.statewide` are computed by
 * `crates/dispersion::median`, which interpolates; anything computed here did not. So `/statewide`
 * said the median district is "$47 per pupil worse off" under the regime counterfactual while
 * `/district/…/taxes` said "$45 worse" — one site, one phrase, two statistics, neither stale and
 * neither a typo. Two definitions in one workspace is the exact defect the crate side already paid
 * for, and a boundary between Rust and TypeScript does not make it a different defect.
 *
 * So: one definition, and it is the crates'. The prose pays the difference. A sentence naming a
 * district as the bearer of an interpolated value — *"the median district here receives $X"* —
 * is now written so it names the statistic instead, because on an even-length series the
 * interpolated median belongs to nobody. That is three sentences of rewriting against eight
 * published figures moving, and it keeps the authority where the rest of this file says it is.
 *
 * Not everything with "median" in its name should call this. `plot/spec.ts`'s box plot takes its
 * quartiles by nearest rank, because a box is *drawn* at an observation and a description reading
 * "median $X" beside a mark placed somewhere else is worse than either convention alone. That one
 * is chart geometry describing itself; this one is a statistic the site publishes.
 *
 * Zero for an empty list, unlike the crate's `Option`. Every caller here interpolates the result
 * into prose or a chart and has no branch for absence; `dispersion::median` returns `None` because
 * on the crate side zero is a plausible dollar figure. The difference is deliberate and is the one
 * place these two disagree.
 */
export function median(values: number[]): number {
  return percentile(values, 0.5);
}

/**
 * The `q`th percentile by linear interpolation on rank `q * (n - 1)`, or zero for an empty list.
 *
 * `crates/dispersion::percentile_sorted`, sorting first. Exported because {@link median} is one
 * value of `q` and having the general form named is what stops the next quantile from being
 * hand-rolled at its call site, which is how this file's subject matter went wrong the first time.
 */
export function percentile(values: number[], q: number): number {
  const sorted = [...values].sort((a, b) => a - b);
  if (sorted.length === 0) return 0;
  if (sorted.length === 1) return sorted[0]!;
  const rank = q * (sorted.length - 1);
  const lo = Math.floor(rank);
  const hi = Math.ceil(rank);
  return lo === hi ? sorted[lo]! : sorted[lo]! + (rank - lo) * (sorted[hi]! - sorted[lo]!);
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
