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
