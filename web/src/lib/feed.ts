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
import { compare } from "./order.ts";
import { median } from "./stats.ts";
import {
  REQUIRED_CONTRACT,
  type Bundle,
  type District,
  type PanelDistrict,
} from "./types.ts";
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
  /** Statewide facts about the property tax base, derived once from the panel. */
  tax: TaxStatewide;
}

/**
 * The property-tax findings the district pages state, computed rather than written down.
 *
 * Every one of these appears on 609 pages. Hard-coding them means a regenerated feed can leave
 * them silently wrong — which nearly happened: the first draft said "two districts are charged
 * more than they spend" and the answer is three.
 */
export interface TaxStatewide {
  /**
   * Districts whose effective Class I rate fell between the two tax years, by floor status.
   *
   * Split by where the district stood at the **start** of the interval, which is not the same as
   * `District.at_millage_floor` — that reports where it stands now. The distinction is not
   * pedantic. Classifying by the end year lets a district that *fell to* the floor be counted as
   * an at-floor district whose rate fell, which is the outcome being measured leaking into the
   * category doing the measuring. It cost this statistic most of its contrast: 21.6% of
   * at-floor districts appeared to have falling rates, against 4.7% under the honest split.
   *
   * Both figures moved when `RATE_FALL_TOLERANCE` was unified (they read 20.5% and 3.6%,
   * computed at a threshold ten times looser than the one this file now defines).
   */
  rateFell: { atFloor: number; aboveFloor: number };
  /** How many districts began TY2023 on each side of the floor. */
  districts: { atFloor: number; aboveFloor: number };
  /** Share of all effective-rate reductions that happen above the floor. */
  reductionsAboveFloor: number;
  /** Median real property charge as a share of operating spending. */
  medianChargeShare: number;
  /** Districts charged more in real property tax than they spend on operations. */
  chargedMoreThanSpent: { name: string; share: number }[];
  /** Above the floor by less than a twentieth of a mill — where the binary stops informing. */
  nearFloor: number;
  /**
   * Districts whose rate crossed 20.0000 between the two tax years, either way.
   *
   * The counterweight to presenting floor status as structural. It is the single most consequential
   * fact about a district's local revenue and, for an eighth of the state, it is also a fact that
   * changed last year and may change back.
   */
  crossedTheFloor: number;
  /**
   * Districts where Taxation's latest effective Class I rate matches Education's to 0.01 mills.
   *
   * Computed rather than written into the copy, on the rule the rest of this block follows: it
   * appeared on the taxes page as the literal `219`, and the profile report's column is a year
   * behind Taxation's latest by construction, so the number moves whenever either publisher does
   * and nothing would have said so. It is the count that makes "the two departments disagree"
   * readable as "one of them has published a later year".
   */
  agreeOnLatest: number;
  /**
   * Share of the latest tax year's taxable value still deferred by recognised valuation.
   *
   * Weighted by Table SD-1's total value, which is the base the charge-off is taken against — a
   * plain mean of the 609 district shares would let a township outweigh Columbus. It was the
   * literal `8.2%`, which is what this computes, and it moves whenever Taxation's staggered
   * county calendar advances: 485 of 609 districts are mid-phase-in, and which ones changes
   * every year by construction.
   */
  deferredShare: number;
  /**
   * The charge-off that deferral removes, in dollars.
   *
   * Deferred value times the charge-off millage, read off each district's own counterfactual
   * rather than restated here. That rate is 23 for all 609 today, so the two orders agree and
   * this is not defending against a difference — it is declining to write a second copy of a
   * number the feed already carries. It was the literal `$793m`.
   */
  deferredChargeOff: number;
  /**
   * The regime difference's median as it stood *before* recognised valuation was corrected.
   *
   * The counterfactual sentence on the taxes page states the correction's effect — the median
   * going from better off under the plan to worse — and both halves of that pair were literals,
   * `$289` and `$45`, with a comment explaining that they were the upper-middle medians and that
   * the feed's own field disagreed with them by $2. It did, and the reason was two definitions of
   * median in one workspace. There is one now, so the "after" half is
   * `statewide.median_regime_difference` and this is the "before".
   *
   * Reconstructed rather than stored: a district's `overstated_by` is exactly the charge-off the
   * deferral removes, so adding it back to `difference` gives the uncorrected comparison. Both
   * halves therefore move together when the panel advances, which is the property the pair of
   * literals could not have.
   */
  medianRegimeDifferenceUncorrected: number;
  /**
   * The enrolled-versus-funded seam, across every district: how far apart the two counts are.
   *
   * The charge-off counterfactual subtracts a deemed local share from a base cost, and the two
   * are not per the same pupil. `crates/regime-diff::at_fy2027` says which counts: the deemed
   * share is `valuation_per_pupil` times the recognized share times the millage, and
   * `valuation_per_pupil` is the profile report's — per **enrolled** ADM, which `denominators.ts`
   * maps to `adm_history[0]`. The base cost it comes off is `base_cost_per_pupil`, which the
   * panel defines as aggregate base cost over **base cost enrolled ADM**, the three-year average
   * carried as `adm`.
   *
   * So the seam is `adm` against `adm_history[0]`, and the paragraph that states it was rendering
   * that ratio per district beside a statewide median of a *different* pair — `adm` against
   * `current_year_adm`, whose median is 1.59% where this one's is 1.95%. A reader was comparing
   * their district against a distribution it was not drawn from. Computed here, once, so the two
   * halves of that sentence cannot come apart again.
   *
   * `max` as well as the median because `denominators.ts` states the same divergence with the
   * same two figures, and its extreme was the only half of it that was right.
   */
  admSeam: { median: number; max: number };
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

  const tax = taxStatewide(bundle.districts);

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
    alphabetical: [...bundle.districts].sort((a, b) => compare(a.name, b.name)),
    valuations: number((d) => d.valuation_per_pupil),
    expenditures: number((d) => d.operating_expenditure_per_pupil),
    tax,
  };
  return cached;
}

/**
 * One district's enrolled-versus-funded gap, as a share, or `null` where it cannot be taken.
 *
 * The per-district half of `TaxStatewide.admSeam`, exported so that the sentence naming both
 * halves computes them from one expression. It read `d.adm / (d.adm_history[0] || d.adm) - 1`
 * inline while the median beside it came from another pair entirely.
 *
 * `null` and not zero for a district with no enrolled count: zero is "the two counts agree", which
 * is a real and different state — and the `|| d.adm` fallback in the expression this replaces
 * produced exactly that, silently, for any district whose enrolled count were absent.
 */
export function admSeamGap(district: District): number | null {
  const enrolled = district.adm_history[0];
  if (enrolled == null || enrolled <= 0) return null;
  return Math.abs(district.adm / enrolled - 1);
}

/**
 * How far two effective Class I rates must differ before one is treated as having fallen.
 *
 * # Why the rule needs a constant at all
 *
 * Because it is applied in two files and was written down three ways. `feed.ts` used `0.0005` to
 * count the statewide split, `tax.ts` used `0.0005` again for the sentence on a district's own
 * page, `schema.spec.ts` used `0.0005` a third time to check the first — and the comment beside
 * the first named **a hundredth of a mill**, which is `0.01` and twenty times larger. At most one
 * of those was the rule. The two surfaces do not currently contradict each other on this data,
 * which is what makes it worth fixing now: it is the same shape as the two definitions of median
 * #100 merged, which also agreed right up until they would not have.
 *
 * # Why this value
 *
 * Table SD-1 publishes effective rates to **four decimal places**, and the feed carries exactly
 * what it publishes — no `class1_rate` in the feed has a fifth. So two rates differ when they
 * differ in the fourth place, and there is nothing finer for a tolerance to discriminate. What is
 * left for it to absorb is the binary representation of a decimal literal, which for a quantity
 * near twenty is a part in `1e15`.
 *
 * Half a unit in the last published place is therefore the **widest** value that cannot discard a
 * difference Taxation actually printed. `0.0005` is ten times that, and it did discard two: a
 * district that went 20.3844 → 20.3840 published a lower rate and was counted as unchanged. The
 * at-floor count is 9 rather than 7 for that reason.
 *
 * The tolerance `beganAtFloor` uses is a different quantity and stays where it is — that is the
 * Rust side's `FLOOR_TOLERANCE`, applied to the statutory floor rather than to a difference.
 */
export const RATE_FALL_TOLERANCE = 0.00005;

/**
 * The statewide property-tax picture, from the two tax years the feed carries.
 *
 * H.B. 920's reduction factors roll an effective rate back as valuation rises and cannot roll it
 * below twenty mills, so the floor decides whether a reappraisal reaches a district's revenue.
 * That is a claim about a mechanism, and with two tax years it is a countable fact instead.
 */
function taxStatewide(districts: District[]): TaxStatewide {
  const rateFell = { atFloor: 0, aboveFloor: 0 };
  const counted = { atFloor: 0, aboveFloor: 0 };
  const shares: { name: string; share: number }[] = [];
  let nearFloor = 0;
  let crossedTheFloor = 0;
  let agreeOnLatest = 0;

  // Recognised valuation, summed over the panel rather than averaged over districts — see
  // `deferredShare`. Both run off the latest tax year, which is the year the charge-off row
  // on the district page is computed at.
  let deferredValue = 0;
  let taxableValue = 0;
  let deferredChargeOff = 0;

  // The regime difference each district would have shown before the deferral was corrected out of
  // the charge-off. See `TaxStatewide.medianRegimeDifferenceUncorrected`.
  const uncorrected: number[] = [];

  // How far each district's funded count sits from its enrolled one. See `TaxStatewide.admSeam`.
  const seam: number[] = [];

  // The floor the Rust side classifies against, restated once rather than at each comparison.
  const FLOOR = 20;

  for (const d of districts) {
    if (d.near_millage_floor) nearFloor++;

    // The last two years, not the ends. The statement this derives — "across TY2023 and TY2024,
    // this many districts saw their rate fall" — is about one interval, and the feed carries four
    // tax years so that recognized valuation can be reconstructed. Reading the ends would silently
    // widen it to TY2021–TY2024 and span a reappraisal.
    const [before, after] = d.property_tax.slice(-2);
    if (before && after && d.property_tax.length >= 2) {
      // Strictly above on one side and at-or-below on the other. Both ends use the same
      // comparison, so a district resting exactly on 20.0000 in both years is not a crossing.
      if (before.class1_rate > FLOOR !== after.class1_rate > FLOOR) crossedTheFloor++;

      // One rule, one constant, one reason — see `RATE_FALL_TOLERANCE`. The comment here named a
      // hundredth of a mill and the code applied a two-thousandth.
      const fell = after.class1_rate - before.class1_rate < -RATE_FALL_TOLERANCE;

      // Where the district stood when the interval opened — see `rateFell`. The tolerance is the
      // Rust side's `FLOOR_TOLERANCE`, applied to the same statutory floor.
      const beganAtFloor = before.class1_rate <= FLOOR + 0.005;
      if (beganAtFloor) {
        counted.atFloor++;
        if (fell) rateFell.atFloor++;
      } else {
        counted.aboveFloor++;
        if (fell) rateFell.aboveFloor++;
      }
    }

    /*
     * Does Education's published rate match Taxation's *latest* year?
     *
     * Mostly it does not, and that is not a disagreement. The profile report's column is
     * `effective_class1_millage_ty23` — a year behind Taxation's latest by construction — so the
     * two agree on the year they share and diverge on the year only one of them has published.
     * Counting the agreement on the latest year is what lets the page say which of those is
     * happening instead of asserting a number somebody typed.
     */
    if (after && d.effective_class1_millage != null) {
      if (Math.abs(after.class1_rate - d.effective_class1_millage) <= 0.01) agreeOnLatest++;
    }

    const regime = d.regime;
    const latest = d.property_tax[d.property_tax.length - 1];
    if (regime?.recognized_share != null && latest) {
      const deferred = (1 - regime.recognized_share) * latest.total_value;
      deferredValue += deferred;
      taxableValue += latest.total_value;
      deferredChargeOff += deferred * (regime.charge_off_mills / 1000);
    }
    if (regime?.difference != null && regime.overstated_by != null) {
      uncorrected.push(regime.difference + regime.overstated_by);
    }

    const gap = admSeamGap(d);
    if (gap != null) seam.push(gap);

    const spending = d.spending_by_function;
    if (after && spending && spending.adm > 0) {
      const operating = spending.operating_per_pupil * spending.adm;
      if (operating > 0) {
        shares.push({ name: d.name, share: after.real_property_taxes_charged / operating });
      }
    }
  }

  const reductions = rateFell.atFloor + rateFell.aboveFloor;

  return {
    rateFell,
    districts: counted,
    reductionsAboveFloor: reductions === 0 ? 0 : rateFell.aboveFloor / reductions,
    // `stats.median`, not a fourth hand-rolled sort. This one took the upper-middle element and
    // so disagreed with every `median_*` field the feed carries — see `stats.ts`.
    medianChargeShare: median(shares.map((s) => s.share)),
    chargedMoreThanSpent: shares
      .filter((s) => s.share > 1)
      .sort((a, b) => b.share - a.share),
    nearFloor,
    crossedTheFloor,
    agreeOnLatest,
    deferredShare: taxableValue === 0 ? 0 : deferredValue / taxableValue,
    deferredChargeOff,
    medianRegimeDifferenceUncorrected: median(uncorrected),
    admSeam: { median: median(seam), max: seam.length === 0 ? 0 : Math.max(...seam) },
  };
}

/**
 * Every district, as `getStaticPaths` wants them.
 *
 * IRN and not a name slug: 28 of the 609 names in this feed are shared by more than one
 * district, so a name-keyed route would collide and silently drop pages.
 */
/**
 * A district's name, qualified by county where the name alone does not identify it.
 *
 * Ohio names school districts after townships, and townships repeat: there are three Green Local
 * districts, three Buckeye Local, three Southern Local. 48 districts share a name with at least
 * one other, and every route that identified a district identified it by bare name — so the
 * title, the `og:title`, the `<h1>` and the compare selects were the same string for two or three
 * different places, and 242 built pages carried a duplicate `<title>`.
 *
 * Qualified only where it is needed. 561 districts are uniquely named and read exactly as before;
 * adding ", Franklin County" to all of them would make every title longer to fix 8% of them.
 *
 * Memoized like the other readers in this file: written once on first call and never mutated
 * after, so the build's 3,488 pages do not each rebuild the tally.
 */
let ambiguousNames: Set<string> | null = null;
/**
 * One district, with everything the funding formula does not read removed.
 *
 * Shared by `/data/panel.json` — the scenario runner's whole-panel copy — and
 * `/data/district/[irn].json`, which `/compare` fetches two of. Two endpoints stripping the same
 * blocks by hand would drift the first time a block was added, and the drift would be invisible:
 * both would still parse.
 *
 * What may be dropped is not a judgement call. {@link PanelDistrict} is the type the formula is
 * written against, so the compiler rejects any use of a field this omits.
 */
export const formulaInputs = ({
    finances: _f,
    outcome: _o,
    base_cost_build_up: _b,
    property_tax: _p,
    spending_by_function: _s,
    millage: _m,
    regime: _r,
    special_education: _se,
    career_technical: _ct,
    english_learners: _el,
    dpia: _d,
    targeted_assistance: _ta,
    gifted: _g,
    house_districts: _hd,
    supplements: _sup,
    transportation: _tr,
    preschool_special_education: _pk,
    transition: _tn,
    national: _nat,
    casino: _cas,
    casino_counties: _cc,
    ...district
  }: District): PanelDistrict => district;

export function qualifiedName(district: District): string {
  if (!ambiguousNames) {
    const counts = new Map<string, number>();
    for (const d of loadFeed().bundle.districts) counts.set(d.name, (counts.get(d.name) ?? 0) + 1);
    ambiguousNames = new Set([...counts].filter(([, n]) => n > 1).map(([name]) => name));
  }
  return ambiguousNames.has(district.name)
    ? `${district.name}, ${district.county} County`
    : district.name;
}

export function districtPaths(): { params: { irn: string }; props: { district: District } }[] {
  return loadFeed().bundle.districts.map((district) => ({
    params: { irn: district.irn },
    props: { district },
  }));
}
