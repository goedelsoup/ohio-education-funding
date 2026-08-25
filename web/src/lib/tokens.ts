/**
 * Whether the stylesheet is on the scale, and by how much it is not.
 *
 * `tokens/` has held type and space values since the design system landed, and `app.css` carried
 * its own literals anyway — `design/README.md` deferred the move because it is "a change with a
 * visual diff", and #112 counted the ledger and closed without spending it. This is what makes the
 * move hold once it has been made.
 *
 * # A value match is not a licence
 *
 * The obvious rule — replace every literal that equals a token's value — is wrong, and measurably
 * so. `.sw { width: 10px }` equals `--radius-lg`, and `h3 { margin: 1.4rem 0 .6rem }` equals
 * `--text-h1`. Adopting either would couple a legend swatch's width to the radius of a card, and a
 * heading's margin to its own font size, so that changing one silently moves the other. A naive
 * pass over `app.css` proposes both.
 *
 * So a token may be adopted only where its CATEGORY matches the property's — {@link CATEGORY} is
 * that table, and it is the whole difference between a scale and a coincidence.
 *
 * # Three outcomes, and only one of them is a defect
 *
 * - **exact** — the literal equals a category-matching token. This is a defect: the token exists,
 *   means this, and is not being used. Held at zero by `tokens.spec.ts`.
 * - **near** — within 20% of a category-matching token. NOT a defect and not adoptable here:
 *   adopting one moves a pixel, and the phase that moved these literals onto the scale is
 *   contractually a no-op. It is a *ratchet* instead — the count may not rise. See
 *   {@link NEAR_MISS_CEILING}.
 * - **none** — no token of that kind is anywhere near. Neither a defect nor a ratchet; some of
 *   these are one-off geometry that should never become a token.
 *
 * The near list is the specification for #186: distinct literal font sizes reaching for a handful
 * of `--text-*` tokens, which is the "no middle register" finding stated in source rather than in
 * a rendered page.
 */

/** Which tokens may be adopted for which properties. Order is irrelevant; prefixes are disjoint. */
export const CATEGORY: Array<{ prefix: RegExp; props: RegExp }> = [
  { prefix: /^--radius-/, props: /^border-radius$/ },
  {
    prefix: /^--space-/,
    props: /^(margin|padding|gap|row-gap|column-gap|inset|top|right|bottom|left)(-\w+)?$/,
  },
  { prefix: /^--text-/, props: /^font-size$/ },
  { prefix: /^--leading-/, props: /^line-height$/ },
  { prefix: /^--tracking-/, props: /^letter-spacing$/ },
  { prefix: /^--weight-/, props: /^font-weight$/ },
  { prefix: /^--measure-/, props: /^max-width$/ },
  { prefix: /^--wrap-/, props: /^max-width$/ },
  { prefix: /^--shadow-/, props: /^box-shadow$/ },
];

/** Properties whose values are on a scale at all. `color` and `display` are not. */
const SCALED =
  /^(font-size|line-height|letter-spacing|font-weight|max-width|border-radius|margin|padding|gap|row-gap|column-gap|inset|top|right|bottom|left|box-shadow)(-\w+)?$/;

/** How close a literal has to be to a token to count as reaching for it rather than as unrelated. */
export const NEAR = 0.2;

/**
 * The near-miss ceiling, which is a ratchet and not a target.
 *
 * 55 declarations sit within 20% of a token they do not use. #186 took every `font-size` and
 * `line-height` among them onto the ramps — those were the register — and left the spacing ones,
 * which are rhythm rather than type and would have doubled that phase's visual diff for a
 * different reason than the one it was about. Every one is a pixel somebody chose
 * by eye next to a value the scale already had an opinion about, and resolving them is #186's job
 * because each resolution moves something.
 *
 * Until then this may not rise. A new literal near a token is a new size chosen by eye, which is
 * exactly how a seven-size scale became the thirteen the page renders.
 */
export const NEAR_MISS_CEILING = 55;

/** A token declaration. */
export interface Token {
  name: string;
  value: string;
}

/** One declaration that could be on the scale and is not. */
export interface Finding {
  line: number;
  property: string;
  value: string;
  /** The literal part of a shorthand this is about — the whole value for a single-part one. */
  part: string;
  token: string;
  tokenValue: string;
  /** How far off, as a fraction of the token's value. Zero for an exact match. */
  distance: number;
}

/** Every `--name: value` on a `:root`-ish block. Order preserved, so the first match wins. */
export function parseTokens(css: string): Token[] {
  return [...css.matchAll(/^\s*(--[a-z0-9-]+)\s*:\s*([^;]+);/gim)].map((m) => ({
    name: m[1] ?? "",
    value: (m[2] ?? "").trim(),
  }));
}

/**
 * Blank out comment bodies while preserving line numbers.
 *
 * `tokens/colors.css` names `--text-body` a dozen times explaining why it was renamed, and
 * `app.css` quotes the pixel values it no longer declares. A rule that read those would be a rule
 * against writing things down — the same reasoning `yearLiterals.spec.ts` already settled.
 */
export function withoutComments(css: string): string {
  return css.replace(/\/\*[\s\S]*?\*\//g, (match) => match.replace(/[^\n]/g, " "));
}

const magnitude = (value: string): [number, string] | null => {
  const match = value.match(/^(-?[\d.]*\.?[\d]+)(rem|px|em|ch)?$/);
  return match == null ? null : [parseFloat(match[1] ?? "0"), match[2] ?? ""];
};

/**
 * Whether two CSS values are the same quantity, which is not whether they are the same string.
 *
 * `.72rem` and `0.72rem` are one value written two ways, and a string comparison calls them
 * different — which reported six declarations of `.8rem` as *near* `--text-sm: 0.8rem` when they
 * are exactly it. The first pass over this stylesheet adopted 67 literals and left those behind,
 * silently, under a rule that looked like it was working.
 *
 * Falls back to string equality for anything that is not a bare number, so `0 10px 28px rgb(...)`
 * still matches `--shadow-panel` the only way it can.
 */
const sameQuantity = (a: string, b: string): boolean => {
  if (a === b) return true;
  const left = magnitude(a);
  const right = magnitude(b);
  return left != null && right != null && left[0] === right[0] && left[1] === right[1];
};

const adoptable = (token: string, property: string): boolean =>
  CATEGORY.some((entry) => entry.prefix.test(token) && entry.props.test(property));

/**
 * Every declaration in `css` that a token of the right category is at or near.
 *
 * Shorthands are examined part by part: `padding: 1.15rem 1.25rem` has one part on the scale and
 * one off it, and reporting the whole declaration as either would be wrong in one direction or the
 * other.
 */
export function audit(css: string, tokens: Token[]): { exact: Finding[]; near: Finding[] } {
  const exact: Finding[] = [];
  const near: Finding[] = [];

  withoutComments(css)
    .split("\n")
    .forEach((line, index) => {
      for (const declaration of line.matchAll(/([a-z-]+)\s*:\s*([^;{}]+)[;}]/g)) {
        const property = (declaration[1] ?? "").trim();
        const value = (declaration[2] ?? "").trim();
        if (value.includes("var(") || !SCALED.test(property) || !/\d/.test(value)) continue;

        const whole = tokens.find((t) => sameQuantity(t.value, value) && adoptable(t.name, property));
        if (whole != null) {
          exact.push({
            line: index + 1, property, value, part: value,
            token: whole.name, tokenValue: whole.value, distance: 0,
          });
          continue;
        }

        for (const part of value.split(/\s+/)) {
          const hit = tokens.find((t) => sameQuantity(t.value, part) && adoptable(t.name, property));
          if (hit != null) {
            exact.push({
              line: index + 1, property, value, part,
              token: hit.name, tokenValue: hit.value, distance: 0,
            });
            continue;
          }
          const measured = magnitude(part);
          /* Zero is excluded deliberately: `margin: 0 0 1.1rem` has two zeros that are not a
             quantity anybody chose, and every token would be "20% away" from them. */
          if (measured == null || measured[0] === 0) continue;
          let closest: Finding | null = null;
          for (const token of tokens) {
            if (!adoptable(token.name, property)) continue;
            const against = magnitude(token.value);
            if (against == null || against[1] !== measured[1] || against[0] === 0) continue;
            const distance = Math.abs(measured[0] - against[0]) / Math.abs(against[0]);
            if (distance === 0 || distance >= NEAR) continue;
            if (closest == null || distance < closest.distance) {
              closest = {
                line: index + 1, property, value, part,
                token: token.name, tokenValue: token.value, distance,
              };
            }
          }
          if (closest != null) near.push(closest);
        }
      }
    });

  return { exact, near };
}

/**
 * The near misses grouped by what they are reaching for, commonest first.
 *
 * The grouping is the point. 114 scattered literals is an inventory; `six declarations write
 * .5rem where --space-3 is 0.55rem` is a decision somebody can make once.
 */
export function clusters(
  near: Finding[],
): Array<{ property: string; part: string; token: string; tokenValue: string; count: number }> {
  const counts = new Map<string, { property: string; part: string; token: string; tokenValue: string; count: number }>();
  for (const finding of near) {
    const key = `${finding.property}|${finding.part}|${finding.token}`;
    const existing = counts.get(key);
    if (existing == null) {
      counts.set(key, {
        property: finding.property, part: finding.part,
        token: finding.token, tokenValue: finding.tokenValue, count: 1,
      });
    } else {
      existing.count += 1;
    }
  }
  return [...counts.values()].sort((a, b) => b.count - a.count);
}
