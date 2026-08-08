/**
 * The feed, read from disk at build time.
 *
 * # Why this module exists, and why it throws
 *
 * The site used to fetch `data/bundle.json` in the browser and render every figure client-side.
 * That kept the feed outside the module graph — regenerating it was a `cargo run` redirect and
 * not a rebuild — and it is the property this module gives up. With real routes, a district's
 * numbers are baked into that district's HTML, so the feed and the build are one artifact and
 * publishing a feed change is a rebuild. In exchange the pages work with JavaScript off, carry
 * their figures to a search engine, and cost one document instead of 1.1 MB.
 *
 * # The verification gate moved here with it
 *
 * `policy.ts` is a second implementation of `crates/project/src/policy.rs`, and the thing that
 * keeps two implementations of one formula honest is that the feed carries Rust-computed
 * checkpoints the TypeScript has to reproduce. That check used to run on every page load and
 * disable the scenario tab when it failed. Disabling a tab is the right answer when the numbers
 * arrive after the page does; it is the wrong answer here, because by the time a baked page is
 * loaded the figures are already printed on it.
 *
 * So the gate runs at build and {@link loadFeed} **throws**, which fails the build. A drifted
 * formula cannot be deployed at all rather than being deployed with one tab held shut. That is
 * strictly stronger, and it is the same check against the same 609-district panel.
 *
 * The two halves stay separate, as they were. A simulation checkpoint that disagrees is a defect
 * and stops the build. A *forecast* checkpoint that disagrees costs the reader the band and
 * nothing else — {@link Feed.forecastable} goes false, the projection cards say why, and the
 * build succeeds. They are different claims and one can be wrong alone.
 *
 * Nothing in here may be imported by client code: it reads the filesystem. The scenario routes,
 * which compute in the browser and therefore still need a runtime gate, fetch the feed over HTTP
 * like the old page did.
 */

import { existsSync, readFileSync } from "node:fs";
import { resolve } from "node:path";

import { BundleSchema } from "./schema/feed.ts";
import { REQUIRED_CONTRACT, type Bundle, type District } from "./types.ts";
import { isForecastVerified, isVerified, verify, type Verification } from "./verify.ts";

/**
 * Where `crates/bundle` writes, and where Astro copies from.
 *
 * Resolved against the working directory rather than `import.meta.url`, which is the obvious
 * choice and the wrong one: Astro bundles this module into `dist/.prerender/` before running it,
 * so at build time `import.meta.url` points into the output tree and the relative path lands two
 * directories from anywhere. The working directory is `web/` under every way this is invoked —
 * `pnpm build`, `pnpm dev`, vitest, and Playwright's web server — and the second candidate covers
 * being run from the repository root instead.
 */
const CANDIDATES = ["public/data/bundle.json", "web/public/data/bundle.json"];
const FEED_PATH =
  CANDIDATES.map((path) => resolve(process.cwd(), path)).find((path) => existsSync(path)) ??
  resolve(process.cwd(), CANDIDATES[0]!);

/** The feed plus the indexes every page would otherwise rebuild over 609 districts. */
export interface Feed {
  bundle: Bundle;
  verification: Verification;
  /** Whether the projection reproduced its checkpoints and a band may be drawn. */
  forecastable: boolean;
  /** By IRN — the identifier the routes are keyed on. */
  byIrn: Map<string, District>;
  /** Alphabetical. 28 district names repeat, so this is not a unique ordering. */
  alphabetical: District[];
  /** Ascending, nulls dropped. For the percentile strips on the district view. */
  valuations: number[];
  /** Ascending, nulls dropped. */
  expenditures: number[];
}

let cached: Feed | null = null;

/**
 * Read, check, and index the feed. Memoized — Astro imports this from ~2,500 pages.
 *
 * @throws if the feed is absent, declares a contract this build does not read, or fails the
 * simulation checkpoints. All three are build-stopping: there is no partial answer worth
 * shipping for any of them.
 */
export function loadFeed(): Feed {
  if (cached) return cached;

  let raw: string;
  try {
    raw = readFileSync(FEED_PATH, "utf8");
  } catch (error) {
    throw new Error(
      `Could not read the feed at ${FEED_PATH}. Regenerate it with:\n` +
        `  cargo run --manifest-path crates/Cargo.toml -p bundle > web/public/data/bundle.json\n` +
        `(${error instanceof Error ? error.message : String(error)})`,
    );
  }

  const json: unknown = JSON.parse(raw);

  // The contract check first, because it produces the better message. A feed from a different
  // contract will also fail the schema, but as a wall of field-level mismatches rather than as the
  // one sentence that explains them.
  const declared = (json as { contract_version?: unknown }).contract_version;
  if (declared !== REQUIRED_CONTRACT) {
    throw new Error(
      `This build reads bundle contract ${REQUIRED_CONTRACT}; the feed declares ` +
        `${String(declared)}. Refusing to build rather than guess at field meanings.`,
    );
  }

  /*
   * Parsed, not cast.
   *
   * This used to be `JSON.parse(raw) as Bundle`, which checks nothing. A field renamed in
   * `crates/bundle` would sail through and reach 609 pages as `undefined` — formatted as an em
   * dash, which looks like a deliberate "not reported" rather than like a defect. The schema is
   * strict, so a field appearing that this build does not know about is caught too: that is the
   * signal the mirror and the Rust struct have drifted, and it is only cheap to act on now.
   */
  const parsed = BundleSchema.safeParse(json);
  if (!parsed.success) {
    const issues = parsed.error.issues
      .slice(0, 12)
      .map((issue) => `  ${issue.path.join(".") || "(root)"}: ${issue.message}`)
      .join("\n");
    const more =
      parsed.error.issues.length > 12
        ? `\n  …and ${parsed.error.issues.length - 12} more`
        : "";
    throw new Error(
      `The feed does not match the shape this build reads, and the build is stopping.\n\n` +
        `${issues}${more}\n\n` +
        `src/lib/schema/feed.ts mirrors the structs in crates/bundle. Either the Rust changed and\n` +
        `this mirror has not, or the feed is corrupt. Regenerate it with:\n` +
        `  cargo run --manifest-path crates/Cargo.toml -p bundle > web/public/data/bundle.json`,
    );
  }
  const bundle: Bundle = parsed.data;

  const verification = verify(bundle);
  if (!isVerified(verification)) {
    const failures = verification.comparisons.filter((c) => !c.agrees);
    const detail =
      verification.comparisons.length === 0
        ? "  the feed carries no checkpoints, so nothing could be checked"
        : failures
            .map((c) => `  ${c.label}: ${c.differences.join("; ")}`)
            .join("\n");
    throw new Error(
      `The formula check FAILED and the build is stopping.\n\n` +
        `src/lib/policy.ts re-derives Ohio's funding formula so the scenario builder does not\n` +
        `need a round trip, and it must reproduce the results crates/project computed before\n` +
        `this site may print any of them. It did not:\n\n${detail}\n\n` +
        `The Rust is authoritative. Either the two implementations have drifted apart or the\n` +
        `feed is from a different build.`,
    );
  }

  const forecastable = isForecastVerified(verification);
  if (!forecastable && bundle.projection) {
    // Not fatal, and deliberately so: a failed forecast check costs the reader the band, not the
    // whole site. It still has to be loud, because a silently missing card reads as a design
    // choice rather than as the defect it is.
    const failures = verification.forecasts.filter((c) => !c.agrees);
    console.warn(
      `\n  The projection check FAILED. Every band is being omitted and the pages say why.\n` +
        (failures.length === 0
          ? `  The feed declares a projection but carries no forecasts to check it against.\n`
          : failures.map((c) => `  ${c.label}: ${c.differences.join("; ")}\n`).join("")),
    );
  }

  const number = (pick: (d: District) => number | null) =>
    bundle.districts
      .map(pick)
      .filter((v): v is number => v != null)
      .sort((a, b) => a - b);

  cached = {
    bundle,
    verification,
    forecastable,
    byIrn: new Map(bundle.districts.map((d) => [d.irn, d])),
    alphabetical: [...bundle.districts].sort((a, b) => a.name.localeCompare(b.name)),
    valuations: number((d) => d.valuation_per_pupil),
    expenditures: number((d) => d.operating_expenditure_per_pupil),
  };
  return cached;
}

/**
 * Every district, as `getStaticPaths` wants them.
 *
 * IRN and not a name slug: 28 of the 609 names in this feed are shared by more than one
 * district, so a name-keyed route would collide and silently drop pages.
 */
export function districtPaths(): { params: { irn: string }; props: { district: District } }[] {
  return loadFeed().bundle.districts.map((district) => ({
    params: { irn: district.irn },
    props: { district },
  }));
}
