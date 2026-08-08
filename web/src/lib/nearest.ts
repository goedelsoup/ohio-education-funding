/**
 * Turning a failed URL into a near miss.
 *
 * A site keyed by six-digit IRN gets typos, and a reader who lands on the 404 almost always had
 * the right district. So the path that failed is worth reading rather than discarding.
 *
 * Pure, and separate from the page that uses it, for a specific reason: the behaviour cannot be
 * exercised end-to-end. A static preview server answers an unmatched path with the front page
 * rather than with `404.html`, so the only place this logic can actually be checked is a unit
 * test — which means it has to be a function that takes a path rather than one that reads
 * `location`.
 */

/** The subset of a search index entry this needs. */
export interface Candidate {
  /** Display title. */
  t: string;
  /** URL. */
  u: string;
  /** For districts, the IRN. */
  a?: string;
}

/**
 * Districts closest to whatever the reader seems to have been asking for.
 *
 * A path containing digits is treated as a mistyped IRN and matched by numeric distance: one
 * wrong digit puts the intended district within a few thousand of the one typed, so the right
 * answer is usually first. A path of words is matched as a substring instead.
 *
 * Returns an empty list rather than a guess when the path carries neither — an empty 404 is better
 * than a confident wrong suggestion, and there is never a redirect either way: silently sending
 * someone to a district they did not ask for would make every number on the page look like it was
 * about theirs.
 */
export function nearest(pathname: string, districts: Candidate[], limit = 6): Candidate[] {
  const digits = pathname.match(/(\d{3,6})/)?.[1];
  if (digits) {
    const target = Number(digits);
    return districts
      .map((entry) => ({ entry, distance: Math.abs(Number(entry.a ?? "0") - target) }))
      .sort((x, y) => x.distance - y.distance || x.entry.t.localeCompare(y.entry.t))
      .slice(0, limit)
      .map((hit) => hit.entry);
  }

  const words = pathname
    .replace(/^\/(district|wiki)\//, "")
    .replace(/[/_-]+/g, " ")
    .trim()
    .toLowerCase();
  if (words.length < 3) return [];
  return districts.filter((entry) => entry.t.toLowerCase().includes(words)).slice(0, limit);
}
