/**
 * `public/_redirects`, read the way Cloudflare Pages reads it.
 *
 * # Why the site needs a reader for its own redirect file
 *
 * Because nothing else serving `dist/` honours it. The host applies `_redirects` at the edge; `vite
 * preview`, which every browser test runs against, serves the file as a static asset and applies
 * none of it — the same blind spot `public/_headers` documents for the CSP. A test that visits
 * `/reach` against the preview gets a 404 whatever the file says, and a test that is told to expect
 * the 404 would pass on a file that redirects nowhere.
 *
 * So the preview server is given this (`scripts/preview.config.ts`), and the old addresses are
 * checked in a browser against the rules that will actually deploy. So is the server the measure
 * instrument loads the build through (`scripts/serve-dist.ts`), which otherwise measured a 404.
 * What it implements is the part of the host's matcher the file uses, written from the documented
 * behaviour:
 *
 * - `:name` matches one path segment and `*` matches the rest, spliced back in as `:splat`;
 * - the status is the third field, 302 when absent;
 * - **the incoming query survives only when the destination has none of its own.** That rule is
 *   the one this site leans on — see `districtScenario` in `routes.ts` — and a matcher that always
 *   carried the query would pass a test the host would fail.
 */

/** One line of `_redirects`. */
export interface Redirect {
  from: string;
  to: string;
  status: number;
}

/**
 * Parse the file: one rule per line, and a line that starts with `#` is a comment.
 *
 * Only a whole line. A `#` inside a rule is a fragment — `/scenario#d=:irn` — and stripping from it
 * to the end of the line, as a shell would, turns that rule into a redirect to `/scenario`.
 */
export function parseRedirects(text: string): Redirect[] {
  const rules: Redirect[] = [];
  for (const raw of text.split("\n")) {
    const line = raw.trim();
    if (line === "" || line.startsWith("#")) continue;
    const [from, to, status] = line.split(/\s+/);
    if (!from || !to) throw new Error(`_redirects: a rule needs a source and a destination: ${raw}`);
    rules.push({ from, to, status: status == null ? 302 : Number(status) });
  }
  return rules;
}

/** A source pattern as a regular expression, with its placeholders' names in capture order. */
function pattern(from: string): { re: RegExp; names: string[] } {
  const names: string[] = [];
  const body = from
    .split("/")
    .map((segment) => {
      if (segment === "*") {
        names.push("splat");
        return "(.*)";
      }
      if (segment.startsWith(":")) {
        names.push(segment.slice(1));
        return "([^/]+)";
      }
      return segment.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
    })
    .join("/");
  return { re: new RegExp(`^${body}$`), names };
}

/**
 * Where a request for `pathname` + `search` is sent, or `null` if no rule matches.
 *
 * The first matching rule wins, as on the host.
 */
export function resolveRedirect(
  rules: readonly Redirect[],
  pathname: string,
  search: string,
): { location: string; status: number } | null {
  for (const rule of rules) {
    const { re, names } = pattern(rule.from);
    const match = re.exec(pathname);
    if (!match) continue;
    let to = rule.to;
    names.forEach((name, i) => {
      to = to.replaceAll(`:${name}`, match[i + 1] ?? "");
    });
    const destination = new URL(to, "https://host.invalid");
    const location = `${destination.pathname}${destination.search || search}${destination.hash}`;
    return { location, status: rule.status };
  }
  return null;
}

/**
 * How many rules without a placeholder the host will read. Cloudflare Pages caps a `_redirects`
 * file at 2,000 static rules and 100 dynamic ones, and past the cap it ignores the rest — silently,
 * at the edge, where no test here can see it.
 */
export const STATIC_RULE_LIMIT = 2000;

/** The keys the site's addresses are built from, read off the feed. */
export interface Addresses {
  irns: readonly string[];
  house: readonly string[];
  senate: readonly string[];
}

/**
 * An address as a person types it, where that differs from the one the site uses.
 *
 * Every key here is zero-padded — an IRN to six digits, a seat in either chamber to three — and the
 * unpadded form is the one a spreadsheet produces from an IRN and the one anyone writes for "House
 * District 90". Every partly padded form too, since `/senate/01` is as natural as `/senate/1`. None
 * for a key with no leading zero, as for Senate 10-33 and the one IRN that starts with a 1: a rule
 * from an address to itself is a redirect loop.
 */
function unpadded(key: string): string[] {
  const forms: string[] = [];
  for (let rest = key; rest.length > 1 && rest.startsWith("0"); ) {
    rest = rest.slice(1);
    forms.push(rest);
  }
  return forms;
}

/**
 * The unpadded addresses (#596), each sent to the page it names: `/house/90` to `/house/090`,
 * `/senate/1` and `/senate/01` to `/senate/001`, `/district/43802` to `/district/043802`.
 *
 * One rule per key rather than a pattern, because the host matches no regular expressions — a
 * `:number` placeholder cannot say "pad me" — and because a static rule costs nothing against the
 * hundred dynamic ones.
 */
export function unpaddedRedirects({ irns, house, senate }: Addresses): Redirect[] {
  const rules: Redirect[] = [];
  const add = (root: string, keys: readonly string[]) => {
    for (const key of keys) {
      for (const form of unpadded(key)) {
        rules.push({ from: `${root}/${form}`, to: `${root}/${key}`, status: 301 });
      }
    }
  };
  add("/district", irns);
  add("/house", house);
  add("/senate", senate);
  return rules;
}

/**
 * The hand-written file with the generated rules after it — what `dist/_redirects` holds.
 *
 * After, so a hand-written rule wins where both match; none does today. Throws rather than writes
 * a file the host would read only the first 2,000 static lines of.
 */
export function withUnpadded(file: string, addresses: Addresses): string {
  const generated = unpaddedRedirects(addresses);
  const rules = [...parseRedirects(file), ...generated];
  const statics = rules.filter((r) => !/[:*]/.test(r.from)).length;
  if (statics > STATIC_RULE_LIMIT) {
    throw new Error(
      `_redirects: ${statics} static rules, over the host's ${STATIC_RULE_LIMIT}; the rest would be ignored`,
    );
  }
  return (
    `${file.replace(/\n*$/, "\n")}\n` +
    "# Written at build by the `unpadded-addresses` integration in `astro.config.mjs`, not by hand.\n" +
    "# An address typed without its leading zeros, sent to the padded one the site uses (#596).\n" +
    generated.map((r) => `${r.from}  ${r.to}  ${r.status}`).join("\n") +
    "\n"
  );
}
