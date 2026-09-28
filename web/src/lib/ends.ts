/**
 * The first and last element of a sequence the caller knows is not empty.
 *
 * `noUncheckedIndexedAccess` types `xs[0]` as possibly undefined, and the answer across this tree
 * had been to switch the check off by hand: `history[0]!` and `history[history.length - 1]!`, the
 * pair written out at every renderer that draws a span. An empty array then yields `undefined`,
 * which the template literal prints as the word, or which fails three property reads later with a
 * message that names neither the array nor the assumption.
 *
 * These make the assumption once and fail at it, so an empty sequence is an error that names the
 * cause rather than a page that says "FYundefined".
 */

export function firstOf<T>(xs: readonly T[]): T {
  if (xs.length === 0) throw new RangeError("firstOf: the sequence is empty");
  return xs[0] as T;
}

export function lastOf<T>(xs: readonly T[]): T {
  if (xs.length === 0) throw new RangeError("lastOf: the sequence is empty");
  return xs[xs.length - 1] as T;
}
