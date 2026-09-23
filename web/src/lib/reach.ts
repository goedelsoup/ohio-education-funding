/**
 * `/reach`: where every district stands, and which of them a lever moves.
 *
 * # The one question the scenario runner cannot answer
 *
 * `/scenario` answers *how much* — a total, a distribution, a ranking of the districts that moved
 * furthest. This answers *which*, and on a formula that pays `max(formula, guarantee)` those are
 * not the same question: a lever can cost the state two hundred million dollars and reach fewer
 * than half the districts in it, and no figure on the other page says so.
 *
 * # Why a chooser and not one fixed pair of axes
 *
 * Because the shape a reader needs depends on what they came to ask. Formula aid against realized
 * aid is the mechanism — the guarantee wall, drawn as the law it is. But *who* is behind that wall
 * is a question about property valuation, poverty and enrolment, and those are only visible when
 * one of them is an axis. Eleven quantities, any pair, is the smallest thing that answers both.
 *
 * What is deliberately not offered is a general chart builder. Every dimension here is a number
 * already on the panel or a ratio of two of them, named, formatted and dated by the feed. None is
 * a fitted quantity and none is computed here for the first time — the rule the rest of this
 * repository's charts follow.
 *
 * # What moves with a lever and what does not
 *
 * `basecost` and every aid dimension are recomputed under the levers. `capacity`, `stateShare`,
 * `valuation`, `poverty` and `adm` are properties of the district as the department published
 * them, and no lever here moves them — the minimum-state-share lever raises a *floor* under the
 * computed share, it does not recompute local capacity. So a district can only travel horizontally
 * on those axes if the other axis is an aid figure, which is exactly the right behaviour and is
 * why the trails read the way they do.
 */

import type { Outcome, Model } from "./policy.ts";
import { MOVED, applyAll, currentLaw, modelOf } from "./policy.ts";
import { count, escapeHtml, money, pct, signedMoney } from "./format.ts";
import { heading } from "./section.ts";
import { renderToString } from "./plot/client.ts";
import { MIN_CLOUD, scatterSpec } from "./plot/spec.ts";
import type { ScatterPoint } from "./chart.ts";
import type { Levers } from "./scenario.ts";
import { LEVER_BOUNDS, defaultLevers, toPolicy } from "./scenario.ts";
import { refreshEffect } from "./refresh.ts";
import { slugify } from "./routes.ts";
import { compare } from "./order.ts";
import type { Panel, PanelDistrict } from "./types.ts";

/** One quantity a district can be placed on. */
export interface Dimension {
  /** The axis label, which is also the option text. */
  label: string;
  format: (v: number) => string;
  /**
   * Draw this axis logarithmically.
   *
   * Declared per dimension rather than decided from the data, because "spans more than a decade"
   * is a property of the quantity and not of one lever position. Honoured only where every value
   * is strictly positive — see `usable` — since a log axis cannot place a zero and half of Ohio's
   * districts have no guarantee at all.
   */
  /**
   * How the fixed frame is chosen for this quantity. See `envelope`.
   *
   * - `zero` — from nothing to a robust high bound. The natural frame for a dollar figure: a
   *   reader reads height off the baseline, and an aid axis starting at $890 exaggerates every
   *   difference on it.
   * - `unit` — 0 to 1, absolutely. A share cannot leave it, so this is the most stable frame there
   *   is and it never needs computing.
   * - `diverging` — symmetric about zero, so that zero is the middle of the axis rather than
   *   wherever the data happens to put it.
   * - `log` — a robust low to a robust high, logarithmically. Only for the two quantities that are
   *   strictly positive by construction and span more than a decade: a 4-pupil island and a
   *   43,000-pupil city are not comparable on a linear axis.
   */
  frame: "zero" | "unit" | "diverging" | "log";
  /**
   * The frame, where the quantity's scale is closed by definition rather than measured.
   *
   * For the department's typology code, which runs 1 through 8 and cannot leave. Measuring it
   * would be both wasteful and wrong: a `zero` frame puts the axis floor at 0, which is the code
   * the department uses for *declining to classify* and has no place on the scale — so the low
   * tick came out blank, because there is no locale to label it with.
   */
  bounds?: [number, number];
  /**
   * The district's value, or `null` where the feed does not carry one.
   *
   * Two of the eleven are nullable in the schema — valuation per pupil and the report card's
   * disadvantaged share — and a district missing one is dropped from the cloud on that axis rather
   * than placed at zero. The card says how many, because a scatter that quietly loses eleven
   * districts is a scatter of a population nobody named.
   */
  of: (d: PanelDistrict, o: Outcome, levers: Levers) => number | null;
}

/**
 * The four locales, at the code each runs from, for the urbanicity axis's tick labels.
 *
 * Names rather than integers, because the code is ordinal and not cardinal: there is no unit in
 * which code 6 is twice code 3, and an axis printing `3` and `6` invites exactly that reading.
 * Only the locale boundaries are labelled — the poverty half of each category is the *secondary*
 * term and putting it on an axis would claim the sequence is ordered by it, which it is not.
 */
const URBANICITY: Record<number, string> = {
  1: "Rural",
  2: "Rural",
  3: "Small town",
  4: "Small town",
  5: "Suburban",
  6: "Suburban",
  7: "Urban",
  8: "Urban",
};

/** Per pupil, on the count the formula pays on, guarding the handful of districts reporting none. */
const per = (value: number, o: Outcome): number => value / (o.adm > 0 ? o.adm : 1);

/**
 * The quantities, in menu order: what the formula does, then who it is doing it to.
 *
 * The order is the argument. The first four are the mechanism — what a district is owed, what it
 * is paid, and the wedge between them — and a reader who opens the menu meets those first. The
 * next four are the district itself, which is what the wedge turns out to correlate with. The last
 * three are outputs of the levers the reader is holding.
 */
export const DIMENSIONS = {
  formula: {
    frame: "zero",
    label: "Formula aid per pupil",
    format: money,
    of: (_d, o) => per(o.formulaAid, o),
  },
  realized: {
    frame: "zero",
    label: "Realized aid per pupil",
    format: money,
    of: (_d, o) => per(o.realizedAid, o),
  },
  guarantee: {
    frame: "zero",
    label: "Guarantee per pupil",
    format: money,
    of: (_d, o) => per(o.guarantee, o),
  },
  dependence: {
    frame: "unit",
    label: "Share of aid from the guarantee",
    format: (v) => pct(v, 0),
    of: (_d, o) => (o.realizedAid > 0 ? o.guarantee / o.realizedAid : 0),
  },
  basecost: {
    frame: "zero",
    label: "Base cost per pupil",
    format: money,
    of: (d, _o, levers) => d.base_cost_per_pupil * levers.baseCostScale,
  },
  capacity: {
    frame: "zero",
    label: "Local capacity per pupil",
    format: money,
    /*
     * What the state charges the district against its own base cost: the complement of the state
     * share, in the same units as the base cost beside it. As published — no lever recomputes it.
     */
    of: (d) => d.base_cost_per_pupil * (1 - stateShare(d)),
  },
  stateShare: {
    frame: "unit",
    label: "State share of base cost",
    format: (v) => pct(v, 0),
    of: (d) => stateShare(d),
  },
  valuation: {
    frame: "log",
    label: "Valuation per pupil",
    format: money,
    of: (d) => d.valuation_per_pupil,
  },
  poverty: {
    frame: "unit",
    label: "Economically disadvantaged",
    format: (v) => pct(v, 0),
    of: (d) => d.economically_disadvantaged,
  },
  adm: {
    frame: "log",
    label: "Enrolled ADM",
    format: (v) => count(Math.round(v)),
    of: (d) => d.current_year_adm,
  },
  urbanicity: {
    frame: "zero",
    bounds: [1, 8],
    label: "How urban the department calls it",
    /*
     * The department's own code as its own label. Not a number a reader should read as a
     * quantity — there is no unit in which code 6 is twice code 3 — so the axis prints the
     * category rather than the integer, and the ticks land on names.
     */
    format: (v) => URBANICITY[Math.round(v)] ?? "",
    /*
     * `null` for the five districts the department removes from the analysis, which is the whole
     * reason this is worth having as an axis rather than a raw code. A `0` here would put three
     * islands at the rural end of a ranking they were excluded from; a null drops them and the
     * card counts them, which is what it already does for a missing valuation.
     */
    of: (d) => (d.typology == null || d.typology.code === 0 ? null : d.typology.code),
  },
  change: {
    frame: "diverging",
    label: "Change per pupil against current law",
    format: signedMoney,
    of: (_d, o) => o.deltaPerPupil,
  },
} as const satisfies Record<string, Dimension>;

export type DimensionKey = keyof typeof DIMENSIONS;

/**
 * The lever fields a preset is compared on, which is every one this page has a control for.
 *
 * `horizon` is not among them: `/reach` draws no band, carries no horizon control, and the value
 * sitting in the levers there is whatever the feed's default was. Comparing it would make every
 * preset read as unpressed on a page where the reader cannot see or change the field that
 * disagrees.
 */
export const PRESET_FIELDS = [
  "guarantee",
  "guaranteeArgument",
  "baseCostScale",
  "minimumStateShare",
  "phaseInGeneral",
  "phaseInDpia",
  "dpiaBlend",
  "supplementalTopRate",
  "transportationFloor",
] as const satisfies readonly (keyof Levers)[];

/** Menu order, which is declaration order — see the note on `DIMENSIONS`. */
export const DIMENSION_KEYS = Object.keys(DIMENSIONS) as DimensionKey[];

/** The published state share of base cost, as a fraction. */
function stateShare(d: PanelDistrict): number {
  return d.aggregate_base_cost > 0 ? d.base_cost_state_share / d.aggregate_base_cost : 0;
}

/**
 * What the dots are coloured by.
 *
 * `type` is the department's own similar-district grouping, and it is drawn as **one group against
 * the rest** rather than as nine colours. That is not a shortcut. `plot/tokens.ts` carries two
 * hues and a three-step ordinal ramp, and the three steps are a measured limit rather than a
 * stylistic one: the ramp's worst pair separates by ΔE 15.0 across normal vision and three
 * dichromacies, and a five-step ramp built to escape that lands at 10.9. Nine categories have no
 * honest encoding in this palette.
 *
 * Highlighting also answers the question better. A reader asking about typology is asking *where
 * do the poor rural districts sit* — they want to find one group in the cloud, not decode nine at
 * once — and a two-hue split says exactly that and nothing else.
 */
export type Shading = "regime" | "change" | "type";

/** The three regimes, in the order the ordinal ramp runs. */
export const REGIMES = [
  "Paid by the formula",
  "Held by the guarantee",
  "Held, and at the minimum state share",
] as const;

/** One of the department's groups, as the picker offers it. */
export interface Group {
  code: number;
  /** The department's short name, read off the feed rather than written here. */
  short: string;
  /** The department's full category name. */
  label: string;
  /** How many districts in this panel are in it. */
  districts: number;
}

/**
 * The department's groups, derived from the panel rather than listed.
 *
 * Every label comes off a district's own `typology` block, so the picker cannot disagree with the
 * cloud about what a code is called, and a category the department reworded arrives here without
 * this file being edited. A code with no district in this panel is simply not offered.
 */
export function groups(districts: Pick<PanelDistrict, "typology">[]): Group[] {
  const found = new Map<number, Group>();
  for (const district of districts) {
    const t = district.typology;
    if (!t) continue;
    const seen = found.get(t.code);
    if (seen) {
      seen.districts += 1;
    } else {
      found.set(t.code, { code: t.code, short: t.short, label: t.label, districts: 1 });
    }
  }
  return [...found.values()].sort((a, b) => a.code - b.code);
}

/**
 * What a selection does to the cloud: light the rest, or remove it.
 *
 * # Two modes because they answer two questions
 *
 * `spotlight` — the default, and the only one until #379 — draws all 609 districts and lets the
 * selection decide which are shaded and trailed. It answers *where do my districts sit in the
 * state*, which is the question a reader arrives with, and it is the only mode that can answer
 * anything at all for a selection of six: see {@link inScope} for why a subset is refused there.
 *
 * `subset` removes the rest. It answers the other question — *how are my districts arranged among
 * themselves* — which the spotlight genuinely cannot, because six marks among six hundred in a
 * 640×420 frame are six dots in a crowd however they are shaded. That is the `/county/[slug]`
 * question, within-county dispersion, asked with a lever in hand.
 *
 * # Why this is not a third axis menu
 *
 * The axis menus change what is *plotted* and the levers change what is *computed*; the page is
 * already careful that a reader must not read one as the other. This is a third category again: it
 * changes who the picture is *about*. So it lives in the scope form with the selection it modifies
 * rather than beside the axes, and it is not offered at all until there is a selection for it to
 * mean something to — inert controls are the same defect the typology picker is hidden for.
 *
 * # What the frame does not do
 *
 * Neither mode refits the axes. `envelope` is measured over all 609 districts across five corner
 * lever runs, and a subset keeps that frame: a county drawn on the state's ruler is comparable with
 * the next county and with itself at another lever position, where one fitted to its own six points
 * would not be. A subset is a smaller population, never a smaller scale.
 */
export type ScopeMode = "spotlight" | "subset";

/** How the cloud is drawn, as against what is being drawn. None of this touches the model. */
export interface View {
  x: DimensionKey;
  y: DimensionKey;
  shading: Shading;
  /**
   * Which typology code is lit, when `shading` is `type`. The department's own code, 0-8.
   *
   * One at a time, for the reason {@link Shading} gives. Default 1 — *Rural, high student poverty*
   * — because it is both the largest group and the one the funding argument is usually about.
   */
  highlight: number;
  /** Draw each district's displacement from its position under current law. */
  trails: boolean;
  /**
   * The counties whose districts the reader is asking about, by slug. Empty is the whole state.
   *
   * See {@link inScope} for what a scope does to the drawing, and why it does not subset it.
   */
  counties: string[];
  /** Districts named one at a time, by IRN. Unioned with `counties`, not intersected with it. */
  districts: string[];
  /**
   * What the selection does to everything outside it. Means nothing where there is no selection.
   *
   * See {@link ScopeMode}. The default is the spotlight, which is the mode that can draw any
   * selection at all.
   */
  mode: ScopeMode;
}

export const DEFAULT_VIEW: View = {
  x: "formula",
  y: "realized",
  shading: "regime",
  highlight: 1,
  trails: true,
  counties: [],
  districts: [],
  mode: "spotlight",
};

/**
 * Read a view out of a query string, falling back per field rather than all-or-nothing.
 *
 * # `ty`, and why it is not `g`
 *
 * The highlight was read from `g`, which is also the guarantee rule's parameter — and `toQuery`
 * wrote both, the second write overwriting the first. So a reader who retired half the guarantee
 * and then coloured the cloud by district type got a URL carrying `g=1`: the code, in place of the
 * rule. Reading it back, `fromQuery` accepts `g` only as one of the four rule names, so `g=1`
 * matched none and the guarantee fell through to `as-enacted`. The lever was silently dropped from
 * every link that page minted, and the picture it drew was plausible.
 *
 * A key per meaning. `g` is the lever's, because it was the lever's first and because a link to a
 * scenario is the thing the runner exists to be able to send someone; `ty` is the highlight's. An
 * old link carrying the overloaded key resolves to the default highlight rather than reading a
 * guarantee rule as a typology code.
 */
export function viewFromQuery(params: URLSearchParams): View {
  const dim = (raw: string | null, fallback: DimensionKey): DimensionKey =>
    raw != null && raw in DIMENSIONS ? (raw as DimensionKey) : fallback;
  const shading = params.get("c");
  /*
   * The raw string first, because `Number(null)` is `0` and `0` is a code the department assigns.
   *
   * This was `Number(params.get(…))` straight into the range check, and a bare `/reach` carries no
   * highlight parameter at all — so the absent case parsed as code 0, which is the department
   * *declining to classify*, cleared `0 <= lit <= 8`, and won against the default. Three districts
   * were lit where the default names the 300-odd rural high-poverty ones, and the page's own prose
   * said otherwise. An empty value is treated the same way: `Number("")` is also 0.
   */
  const named = params.get("ty");
  const lit = named ? Number(named) : NaN;
  return {
    x: dim(params.get("vx"), DEFAULT_VIEW.x),
    y: dim(params.get("vy"), DEFAULT_VIEW.y),
    shading:
      shading === "change" || shading === "regime" || shading === "type"
        ? shading
        : DEFAULT_VIEW.shading,
    // Held to the codes the department assigns. A query string naming code 11 would otherwise
    // light nothing and look like a page that had lost its highlight.
    highlight:
      Number.isInteger(lit) && lit >= 0 && lit <= 8 ? lit : DEFAULT_VIEW.highlight,
    // Absent means the default, which is on. Only an explicit `0` turns them off.
    trails: params.get("t") !== "0",
    /*
     * The scope, validated for *shape* here and against the panel in `inScope`.
     *
     * Two stages because this function has no panel to check against, and inventing one would make
     * a parser depend on a feed. A slug or an IRN that no district carries is simply not in the
     * union — which is the same answer a reader gets for a county the department reattributed, and
     * is why it is dropped there rather than rejected here.
     *
     * Deduplicated, because `?co=allen,allen` is a set written twice and the chips are a set.
     */
    counties: list(params.get("co"), (v) => /^[a-z0-9-]+$/.test(v)),
    districts: list(params.get("d"), (v) => /^\d{6}$/.test(v)),
    /*
     * A key of its own, like every other meaning on this page — see the account above of what
     * happened when the highlight shared the guarantee rule's `g`.
     *
     * Absent is the spotlight, and only an explicit `1` is the subset, which is the rule `t` uses
     * in the other direction. It is read whether or not a scope came with it: a `?only=1` with no
     * selection draws exactly what a bare `/reach` draws, because `inScope` returns no scope and a
     * mode with nothing to subset subsets nothing. Held that way rather than rejected here so the
     * parser stays a parser, the way an unmatched county slug is.
     */
    mode: params.get("only") === "1" ? "subset" : DEFAULT_VIEW.mode,
  };
}

/** One comma-separated query parameter as a set of well-shaped values, in the order given. */
function list(raw: string | null, shaped: (value: string) => boolean): string[] {
  if (raw == null) return [];
  return [...new Set(raw.split(",").map((v) => v.trim()).filter((v) => v !== "" && shaped(v)))];
}

/** One county, as the scope picker offers it. */
export interface ScopeCounty {
  slug: string;
  /** The department's own attribution, as the feed spells it. */
  name: string;
  /** How many districts in this panel it holds. */
  districts: number;
}

/**
 * The counties, derived from the panel rather than listed.
 *
 * Eighty-eight of them, and none written here: a county the department reattributes a district to
 * arrives in the control without this file being edited, and the slug comes from the same
 * `slugify` that builds `/county/…` so the two cannot disagree about what a county is called.
 *
 * Ordered by name, because that is how a reader looks for one. `counties()` in `county.ts` orders
 * by district count for a different page with a different question.
 */
export function scopeCounties(districts: Pick<PanelDistrict, "county">[]): ScopeCounty[] {
  const found = new Map<string, ScopeCounty>();
  for (const district of districts) {
    const seen = found.get(district.county);
    if (seen) seen.districts += 1;
    else found.set(district.county, { slug: slugify(district.county), name: district.county, districts: 1 });
  }
  return [...found.values()].sort((a, b) => compare(a.name, b.name));
}

/**
 * The counties that hold enough districts to be drawn on their own.
 *
 * The measurement the subset mode turns on, and the one every sentence about it argues from. A
 * county with fewer than {@link MIN_CLOUD} districts cannot be a cloud, so asking for it alone gets
 * a refusal rather than a picture — and the refusal has to be able to say how rare that is without
 * a number typed into the prose. On this feed it is nine of eighty-eight.
 *
 * Counted off the panel for the reason {@link scopeCounties} is: a county the department
 * reattributes a district to moves this figure, and a page that spelled it out would go quietly
 * stale. It is a *necessary* condition and not a sufficient one — a district with no valuation is
 * not a drawn point, so a county of twelve can still fall under the floor on the axes actually
 * chosen, and `renderReach` counts what it drew rather than trusting this.
 */
export function cloudCounties(districts: Pick<PanelDistrict, "county">[]): ScopeCounty[] {
  return scopeCounties(districts).filter((county) => county.districts >= MIN_CLOUD);
}

/**
 * Which districts the reader is asking about, or `null` for all of them.
 *
 * # Why a scope lights by default, and subsets only on request
 *
 * Because a filter that drew only the selection would draw nothing at all for most of Ohio.
 * `scatterSpec` refuses fewer than twelve points — *"a scatter of three districts would read as a
 * finding about a population that has not been measured"* — and a null spec renders to the empty
 * string, so the card would come out as a heading, a legend and nothing between them. Counted
 * against this feed: 88 counties, 609 districts, a median of 6 districts per county, and **only 9
 * counties hold 12 or more**. Four hold exactly one. A subsetting filter would be blank for 79 of
 * 88 counties, and for every selection of a single district.
 *
 * The frame is the second reason and the stronger one. `envelope` is measured over all 609
 * districts across five corner lever runs precisely so that a district which did not move looks
 * like a district which did not move. A scope that reached it would refit the axes to a county and
 * reintroduce the moving ruler this route was rebuilt to remove — invisibly, because a refitted
 * axis looks like an axis.
 *
 * So the whole state stays drawn and the scope decides what is *lit*: in-scope districts keep their
 * shading and their trails, out-of-scope districts are muted and draw none. That is also the move
 * this page already makes for the typology, deliberately — one group against the rest rather than
 * nine hues — and it keeps the shape of the state, which is the thing the cloud says at rest.
 *
 * What the second mode changes is only that: {@link ScopeMode}`.subset` removes the context rather
 * than muting it, for the nine counties and the hand-built selections that hold enough districts to
 * be a cloud on their own — and `renderReach` refuses below {@link MIN_CLOUD} with the reason
 * stated rather than drawing a card with a heading, a legend and nothing between them. The frame
 * does not move either way, which is why a subset is still a comparable picture. Neither the
 * argument above nor the set this function computes depends on the mode: this is the same set of
 * districts, drawn against a different background. See {@link cloudCounties} for which counties can
 * support the second mode at all.
 *
 * # Union, not intersection
 *
 * The department attributes each district to exactly one county, so intersecting a county with a
 * district would make *Cuyahoga and Upper Arlington* empty. A scope is a set of districts arrived
 * at two ways: every district in a named county, plus every district named outright.
 *
 * # An unresolvable selection is no selection
 *
 * `null` where nothing matched, which draws the whole state rather than an empty cloud. The control
 * cannot produce such a selection — every option comes off this panel — so this is the query-string
 * path: `?co=nowhere` is a link to a county that does not exist, and the honest answer to it is the
 * page a bare `/reach` draws.
 */
export function inScope(
  districts: Pick<PanelDistrict, "irn" | "county">[],
  view: Pick<View, "counties" | "districts">,
): Set<string> | null {
  if (view.counties.length === 0 && view.districts.length === 0) return null;
  const wantedCounties = new Set(view.counties);
  const wantedDistricts = new Set(view.districts);
  const scope = new Set<string>();
  for (const district of districts) {
    if (wantedCounties.has(slugify(district.county)) || wantedDistricts.has(district.irn)) {
      scope.add(district.irn);
    }
  }
  return scope.size > 0 ? scope : null;
}

/**
 * The frame, fixed across everything the levers can reach — and robust to the four-pupil island.
 *
 * # Why the axes must not be fitted to the run being drawn
 *
 * This chart is redrawn on every slider tick, and an axis fitted to its own points refits with
 * them. Drag base cost and the cloud holds still while the *frame* moves underneath it — which is
 * precisely the reading a movement chart must not produce, because a reader cannot tell whether
 * the districts moved or the ruler did. It is also what made the trails hard to read: a segment
 * between two positions only means something if the two are measured on one scale.
 *
 * # Why it is not the extremes either
 *
 * The first version of this took the true minimum and maximum over the corner runs, and the answer
 * was −$56 to $147,555 — a frame in which every district in Ohio is a smudge against the left edge.
 * **Kelleys Island Local has four pupils.** Its aid per pupil is $26,700 against a 99th percentile
 * of $15,305, and its base cost per pupil is $371,449 against a 99th percentile of $12,221. A
 * denominator of four makes a per-pupil figure a different kind of quantity, and sizing the state's
 * axis to it discards the state to show one island.
 *
 * So the bound is a **quantile**, not a maximum, and the marks are clipped to the frame. The
 * handful of districts beyond it are counted and named in the card rather than being allowed to
 * set the scale for the other six hundred — the same trade `censored cells` and the outlier rules
 * elsewhere in this repository already make.
 *
 * # The lower bound is zero where zero is the floor
 *
 * A dollar figure is read as a height above the baseline, so an aid axis starting at $890 makes
 * every difference on it look larger than it is. Shares get 0 to 1 absolutely, which is the most
 * stable frame there is and needs no computing. The two log dimensions are the exception in both
 * directions: they are strictly positive by construction, and a 4-pupil district beside a
 * 43,000-pupil one is not a linear comparison.
 *
 * # Corners, not a proof
 *
 * The quantile is taken per corner run and the widest is kept, so a setting at the edge of the
 * controls is inside the frame rather than pushing it. The corners are not a proof that nothing
 * escapes — the lever space has five axes — so a setting that reached past them would clip a few
 * more districts than usual, and the card would say so. That is the failure this is built to have:
 * a visible count, not a moving ruler.
 *
 * Cached on the panel, which is one object for the life of the page — the arrangement
 * `refreshEffect` uses and for the same reason.
 */
const envelopes = new WeakMap<object, Map<DimensionKey, [number, number]>>();

/**
 * The share of districts the frame is sized to hold.
 *
 * 0.99, which on 609 districts leaves about six outside it. Lower and the frame starts cutting off
 * districts a reader would reasonably look for; higher and Kelleys Island is back.
 */
export const FRAME_QUANTILE = 0.99;

/** The value at `q` of a sorted copy. Nearest-rank, which needs no interpolation to explain. */
function quantile(values: number[], q: number): number {
  const sorted = [...values].sort((a, b) => a - b);
  return sorted[Math.min(sorted.length - 1, Math.max(0, Math.floor(q * (sorted.length - 1))))] ?? 0;
}

/**
 * Lever settings at the edges of the reachable space.
 *
 * Chosen to bound the two quantities that actually set the frame — what the formula computes and
 * what the district is paid — from both directions at once. The two opposite corners, plus the two
 * settings that pull formula aid and realized aid apart: base cost at its ceiling with the
 * guarantee gone, which maximises the formula and removes the floor under it, and base cost at its
 * floor with the guarantee intact, where the guarantee is carrying the most districts.
 */
function corners(model: Model): Levers[] {
  const law = defaultLevers(model);
  const b = LEVER_BOUNDS;
  return [
    law,
    {
      ...law,
      guarantee: "removed",
      baseCostScale: b.baseCostScale.min,
      minimumStateShare: b.minimumStateShare.min,
      phaseInGeneral: b.phaseInGeneral.min,
      phaseInDpia: b.phaseInDpia.min,
    },
    {
      ...law,
      baseCostScale: b.baseCostScale.max,
      minimumStateShare: b.minimumStateShare.max,
      phaseInGeneral: b.phaseInGeneral.max,
      phaseInDpia: b.phaseInDpia.max,
    },
    { ...law, guarantee: "removed", baseCostScale: b.baseCostScale.max },
    { ...law, baseCostScale: b.baseCostScale.min },
  ];
}

/** The fixed domain for one dimension, or `null` where it carries no value anywhere. */
export function envelope(panel: Panel, key: DimensionKey): [number, number] | null {
  let cached = envelopes.get(panel);
  if (!cached) {
    cached = new Map();
    const model = modelOf(panel.statewide);
    const runs = corners(model).map((levers) => ({
      levers,
      outcomes: applyAll(panel.districts, toPolicy(levers), model),
    }));
    for (const dimension of DIMENSION_KEYS) {
      // Widened to the interface: `satisfies` keeps each entry's literal type, on which an
      // optional field it does not set does not exist.
      const dim: Dimension = DIMENSIONS[dimension];
      const stated = dim.bounds;
      if (stated) {
        cached.set(dimension, stated);
        continue;
      }
      if (dim.frame === "unit") {
        cached.set(dimension, [0, 1]);
        continue;
      }
      let low = Infinity;
      let high = -Infinity;
      for (const run of runs) {
        const values: number[] = [];
        for (const [i, o] of run.outcomes.entries()) {
          const v = dim.of(panel.districts[i]!, o, run.levers);
          if (v != null && Number.isFinite(v)) values.push(v);
        }
        if (values.length === 0) continue;
        /* The widest run wins each end, so a setting at the edge of the controls sits inside the
           frame rather than pushing it. */
        if (dim.frame === "diverging") {
          const magnitude = quantile(values.map(Math.abs), FRAME_QUANTILE);
          low = Math.min(low, -magnitude);
          high = Math.max(high, magnitude);
        } else {
          high = Math.max(high, quantile(values, FRAME_QUANTILE));
          low =
            dim.frame === "zero"
              ? 0
              : Math.min(low, quantile(values, 1 - FRAME_QUANTILE));
        }
      }
      if (low <= high && Number.isFinite(low) && Number.isFinite(high)) {
        cached.set(dimension, [low, high]);
      }
    }
    envelopes.set(panel, cached);
  }
  return cached.get(key) ?? null;
}

/**
 * A named lever position.
 *
 * The artifact this page grew out of hard-coded six of these as multipliers. Two of them are not
 * expressible here and they are the two worth naming: `1.0395` is the refresh a *draft in the feed*
 * prices, so it is read from the feed rather than typed, and `2.65` — the scale at which Delphos
 * City finally clears its guarantee — is outside `LEVER_BOUNDS.baseCostScale`, so offering it as a
 * button would mint a position the sliders cannot represent and the query string cannot carry.
 * That a preset is unreachable is itself the finding, and it belongs in prose rather than in a
 * control that would have to lie about where the slider is.
 */
export interface Preset {
  /** Stable name, used as the button's value and in the URL. */
  key: string;
  label: string;
  /** What this setting *is*, on the button's second line. Never decoration. */
  note: string;
  /**
   * The whole lever state, not the fields this preset is about.
   *
   * These were partial at first — a preset naming only what it changed, so two could compose — and
   * the row then reported three of them pressed at once, because each was still true of the fields
   * it had named. Under a heading reading "Start from", three simultaneous selections is a lie
   * about what the reader is looking at. A preset is a position, one at a time, and a reader who
   * wants base cost *and* a phased guarantee reaches the sliders, which is what they are for.
   */
  levers: Levers;
}

/**
 * The presets, derived rather than written.
 *
 * `refreshEffect` finds the base-cost provision the feed actually prices and returns its scale, so
 * "Fund the plan" is the bill's own number — and where the feed prices no refresh, the button is
 * not offered at all rather than being offered at an invented scale.
 */
export function presets(panel: Panel): Preset[] {
  const model = modelOf(panel.statewide);
  const refresh = refreshEffect(panel.districts, panel.drafts, model);
  const law = defaultLevers(model);
  const from = (over: Partial<Levers>): Levers => ({ ...law, ...over });
  const list: Preset[] = [
    {
      key: "law",
      label: "Current law",
      note: "the enacted act, which is this page's zero",
      levers: law,
    },
  ];
  if (refresh) {
    list.push({
      key: "refresh",
      label: "Fund the plan",
      note: `base cost at the refresh the feed prices, ${refresh.scale.toFixed(4).replace(/0+$/, "")}×`,
      levers: from({ baseCostScale: refresh.scale }),
    });
  }
  list.push(
    {
      key: "ceiling",
      label: "Base cost at the ceiling",
      note: `${LEVER_BOUNDS.baseCostScale.max.toFixed(2)}×, the furthest this page's slider goes`,
      levers: from({ baseCostScale: LEVER_BOUNDS.baseCostScale.max }),
    },
    {
      key: "retire",
      label: "Retire half the floor",
      note: "the guarantee phased out, half of it retained",
      levers: from({ guarantee: "phase-out", guaranteeArgument: 0.5 }),
    },
    {
      key: "floor",
      label: "Minimum share at 25%",
      note: "raise the floor under the state share instead of the base cost",
      levers: from({ minimumStateShare: 0.25 }),
    },
  );
  return list;
}

/**
 * What the reader asked about, in words.
 *
 * The scope has to be *named* wherever a count is stated against it. "4 of 6 districts are paid the
 * same" is a sentence about a population, and a reader who has forgotten which six — or who has
 * been sent the link by somebody else — is reading a statistic with no subject. The chips above the
 * plot say what is selected; this is what says it inside the claim.
 *
 * Counties by name and districts by number. Three counties are named outright and the fourth turns
 * the phrase into a count, because a legend entry listing eleven counties is a paragraph. A district
 * named individually *and* covered by a selected county is already in that county's count and is
 * not counted twice.
 */
function scopeLabel(
  districts: Pick<PanelDistrict, "irn" | "county">[],
  view: Pick<View, "counties" | "districts">,
): string {
  const wantedCounties = new Set(view.counties);
  const wantedDistricts = new Set(view.districts);
  const named = [...new Set(districts.map((d) => d.county))]
    .filter((name) => wantedCounties.has(slugify(name)))
    .sort(compare);
  const parts: string[] = [];
  if (named.length > 0) {
    parts.push(
      named.length <= 3
        ? `${named.join(", ")} ${named.length === 1 ? "County" : "counties"}`
        : `${named.slice(0, 3).join(", ")} and ${count(named.length - 3)} more counties`,
    );
  }
  const alone = districts.filter(
    (d) => wantedDistricts.has(d.irn) && !wantedCounties.has(slugify(d.county)),
  ).length;
  if (alone > 0) {
    parts.push(`${count(alone)} district${alone === 1 ? "" : "s"} named on its own`);
  }
  /* Only where nothing resolved, which `inScope` turns into no scope at all — so this is the
     phrase no caller should ever be able to print. Stated rather than left as an empty string. */
  return parts.length === 0 ? "the selection" : parts.join(", plus ");
}

/**
 * Every district in one picture, and the wall it is paid against.
 *
 * # The geometry, where the axes are the default pair
 *
 * Plotted as formula aid against realized aid, the mechanism stops being a sentence and becomes a
 * shape. No district can sit below the identity line, because no district is paid less than its
 * formula amount. The ones **on** it are paid by the formula and travel up it. The ones **above**
 * it are held, and the vertical distance is their guarantee. Pull base cost and the held slide
 * sideways — their formula amount grows, their payment does not — until they reach the line and
 * start to climb. The corner is the mechanism.
 *
 * The identity line is drawn for that pair and for no other. It asserts `y ≥ x` as a law, which is
 * true of realized against formula aid and is not true of any other pair the menu can make — so it
 * is gated on the pair rather than on a flag a caller could set wrongly.
 *
 * # The trails are the finding, including the ones that are not there
 *
 * Each district is drawn from where current law puts it to where these levers do, and the trails
 * that matter are the **flat** ones: a held district's formula amount rises with base cost while
 * its payment does not, so it travels right along a horizontal line and gains nothing. A cloud of
 * after-positions would place those districts where they always were and say nothing about it.
 *
 * # Three bands rather than two series
 *
 * The regime shading needs a third step: held *and* at the minimum state share, whose local
 * capacity is censored and which therefore cannot move at all until the floor itself does. That is
 * an ordered fact — unconstrained, held, held twice over — so it takes the ordinal ramp. The other
 * shading is a genuine two-way split and takes the validated pair, where `formula` is the hue
 * `SERIES` documents as doubling for "gain".
 *
 * # The scope, which changes what is lit and — on request — what is drawn
 *
 * See {@link inScope} for why the default lights rather than subsets. Under the spotlight every
 * district stays in the cloud; an out-of-scope one is muted, loses its shading and draws no trail.
 * What that costs is prose: every count a reader reads as an *answer* has to be restated against
 * the scope, because "253 of 609 are paid the same" is a different claim from "4 of 6 in Athens
 * County are". The two counts that stay global under the spotlight are the clipped and the
 * unplaced — both are facts about the drawing, and all 609 are still drawn.
 *
 * Under {@link ScopeMode}`.subset` the rest of the state is not drawn at all, and then *those two
 * become scope-relative as well*. That is the whole of what the second mode costs here, and it is
 * the part worth being careful about: the unplaced note explains why a district is missing — no
 * valuation, no report-card share, no typology assigned — and under a subset it must not have to
 * compete with a fourth reason that is only "you did not select it". It does not, because an
 * out-of-scope district is dropped before the axes are read, so it is neither drawn nor counted as
 * unplaced, and the denominator is the selection.
 *
 * # The refusal
 *
 * A subset that drew fewer than {@link MIN_CLOUD} points would render to the empty string —
 * `scatterSpec` returns `null` there and `renderToString` draws nothing — so the card would come
 * out as a heading, a legend and a gap. It refuses instead, in that rule's own terms, and sends the
 * reader to the spotlight, which can draw any selection.
 *
 * # The identity line
 *
 * Unchanged, and deliberately. It is a law — a district is paid the larger of its formula amount
 * and its guarantee — so it holds for six districts exactly as it holds for six hundred, and it is
 * the one mark on this chart that is not data. A subset gets it on the same gate as the whole
 * state: the pair of axes, not the population.
 */
export function renderReach(panel: Panel, levers: Levers, view: View, chip = ""): string {
  const model = modelOf(panel.statewide);
  const outcomes = applyAll(panel.districts, toPolicy(levers), model);
  const law = applyAll(panel.districts, currentLaw(model), model);

  const dx = DIMENSIONS[view.x];
  const dy = DIMENSIONS[view.y];
  const wall = view.x === "formula" && view.y === "realized";

  const zero = defaultLevers(model);
  const scope = inScope(panel.districts, view);
  /*
   * Whether the rest of the state is removed rather than muted.
   *
   * Gated on there *being* a scope, not on the mode alone. A mode with nothing to subset is not a
   * subset of anything — `?only=1` on a bare `/reach` is the whole state — and deriving it here
   * once keeps every branch below from having to remember that.
   */
  const subset = scope != null && view.mode === "subset";
  const points: ScatterPoint[] = [];
  let unplaced = 0;
  let pinned = 0;
  /* Drawn and in scope, which is the denominator of every count a reader reads as an answer. */
  let scoped = 0;

  for (const [i, o] of outcomes.entries()) {
    const d = panel.districts[i]!;
    const inside = scope == null || scope.has(d.irn);
    /*
     * Dropped before its coordinates are read, which is what keeps the unplaced count honest.
     *
     * Under a subset a district outside the selection is not drawn, and it must not be counted as
     * *not drawn* either: the note below that accounts for the undrawn is about districts the feed
     * carries no figure for, and a county's worth of selections would otherwise bury that in six
     * hundred districts the reader simply did not ask about.
     */
    if (subset && !inside) continue;
    // Falls back to the district's own position, which draws no trail — the honest answer if the
    // two runs ever disagreed about the population, rather than a silently missing segment.
    const before = law[i] ?? o;
    const x = dx.of(d, o, levers);
    const y = dy.of(d, o, levers);
    if (x == null || y == null) {
      unplaced += 1;
      continue;
    }
    const point: ScatterPoint = {
      x,
      y,
      hover: `${d.name}: ${dx.label.toLowerCase()} ${dx.format(x)}, ${dy.label.toLowerCase()} ${dy.format(y)} — ${
        o.onGuarantee
          ? o.atMinimumStateShare
            ? "held, and at the minimum state share"
            : "held by the guarantee"
          : "paid by the formula"
      }${Math.abs(o.delta) > MOVED ? `, ${signedMoney(o.deltaPerPupil)} per pupil` : ", unmoved"}`,
    };
    /*
     * Out of scope is drawn as context: muted, unshaded, and with no trail.
     *
     * Unshaded rather than shaded-and-muted because the shading answers a question that was asked
     * about a different population. A muted ordinal band would still be saying *this district is
     * held twice over* in a legend the reader is reading about their own county.
     *
     * The trail is withheld here rather than drawn faintly, which is why `muted` has to reach the
     * dot: 600 faint segments across a cloud of six is not context, it is the cloud back again.
     */
    if (!inside) {
      point.muted = true;
      points.push(point);
      continue;
    }
    scoped += 1;
    if (view.shading === "regime") {
      point.band = o.onGuarantee ? (o.atMinimumStateShare ? 2 : 1) : 0;
    } else if (view.shading === "type") {
      // One group lit, everything else left neutral. See `Shading` for why this is not nine hues.
      if (d.typology?.code === view.highlight) point.series = "formula";
    } else if (o.delta > MOVED) {
      point.series = "formula";
    } else if (o.delta < -MOVED) {
      point.series = "guarantee";
    }
    if (view.trails) {
      /* Under current law both runs are the same run, so the levers produce no `from` that
         differs — which is correct, and is why the page reads at rest as a still rather than as
         an empty one. A before-position that is itself unplaceable draws no trail rather than
         anchoring the segment at an invented coordinate. */
      const fx = dx.of(d, before, zero);
      const fy = dy.of(d, before, zero);
      if (fx != null && fy != null) point.from = { x: fx, y: fy };
    }
    points.push(point);
    if (Math.abs(o.delta) <= MOVED) pinned += 1;
  }

  /*
   * Unreached means the **payment** did not move, which is the vertical axis alone when the wall
   * is drawn — and is the district's own delta on every other pair.
   *
   * Not both coordinates. Base cost scales what the formula computes for every district including
   * the held ones, so a held district's x moves whatever happens, and counting a district as
   * pinned only when neither coordinate moved would report almost nobody as pinned on the one
   * lever this page exists to explain. What a held district draws is a trail that runs flat.
   */
  const placed = points.length;

  /* What the reader asked about, in words, for the notes and the chart's spoken label. */
  const asked = scope == null ? "" : scopeLabel(panel.districts, view);
  /*
   * The figures the scope notes argue from, read off the panel rather than written into the
   * sentence. A county the department reattributes moves them; a feed that gained a county moves
   * them; and `MIN_CLOUD` is the form's own floor rather than a second copy of it.
   */
  const allCounties = scope == null ? [] : scopeCounties(panel.districts);
  const enough = scope == null ? [] : cloudCounties(panel.districts);

  /*
   * Who every count on this card is about.
   *
   * The state under the spotlight, because all 609 are drawn there and a denominator of 609 is what
   * a reader can check against the picture. The selection under a subset, because nothing else is
   * on the chart — "3 of 609 districts are not drawn" beside a cloud of Athens County would be a
   * sentence about a population the reader removed on purpose.
   */
  const population = subset ? scope.size : panel.statewide.districts;

  /*
   * The refusal, and why it is a card rather than a smaller cloud.
   *
   * `scatterSpec` returns `null` under `MIN_CLOUD` points and `renderToString` draws nothing for a
   * null spec, so the alternative to saying this is a heading with a gap under it. The floor is not
   * a rendering limit and the refusal says so in the rule's own words: at six marks the eye still
   * takes a shape off the cloud, and the shape it takes is whichever districts happen to be in the
   * county rather than anything that was measured.
   *
   * Counted on what was actually drawn, not on the size of the selection. A county of twelve whose
   * axes include valuation per pupil can still fall under the floor, because a district the feed
   * carries no valuation for is not a point — so the two figures are stated separately where they
   * differ, or the reader is told a number the picture does not have.
   *
   * The spotlight is the way out, and it is offered as a control rather than described: it can draw
   * this selection, and it is one click from here. `data-scope-mode` is delegated from
   * `#scenario-out` in `scripts/scenario.ts`, because this card is replaced on every lever tick.
   */
  if (subset && placed < MIN_CLOUD) {
    return `
    <div class="card stage" id="positions" data-part="positions">
      <h2>${heading("positions", "Where every district stands", chip)}</h2>
      <p class="note"><strong>${escapeHtml(asked)} is too small to draw on its own.</strong> ${
        placed === scoped
          ? `${count(scoped)} district${scoped === 1 ? " is" : "s are"} in the selection`
          : `${count(scoped)} districts are in the selection and ${count(placed)} of them carry a
             value on both axes`
      }, against the ${MIN_CLOUD} points this form will not go below.</p>
      <p class="note">That floor is not a limit of the drawing. ${
        placed === 1
          ? "A single mark still reads as a position"
          : `A scatter of ${count(placed)} marks still reads as a shape`
      }, and what it reads as is whichever districts happen to be in the selection — a finding
        about a population nobody measured. Only <strong>${count(enough.length)} of
        Ohio's ${count(allCounties.length)} counties</strong> hold ${MIN_CLOUD} districts or more,
        which is why this is the mode that has to be asked for.</p>
      <p class="note">The other view answers what this selection can support: every district drawn,
        ${escapeHtml(asked)} lit, on the frame the whole state is measured on — which is where a
        position becomes legible in the first place. <button type="button" class="linkish"
        data-scope-mode="spotlight">Light the selection instead</button>, or select more
        districts.</p>
    </div>`;
  }

  /*
   * The fixed frame. See `envelope`: the axes are the same on every lever position, so a district
   * that did not move looks like a district that did not move.
   */
  const xFixed = envelope(panel, view.x);
  const yFixed = envelope(panel, view.y);

  /*
   * The scale is a property of the quantity, not of the run.
   *
   * Asking the data would make the scale itself a function of the lever: a setting under which
   * every district happens to be paid something draws a log axis, and the next tick, where one is
   * paid nothing, draws a linear one — a chart that changes what kind of chart it is while the
   * reader is dragging a slider. The two log dimensions are strictly positive by construction, and
   * the envelope's low end is checked anyway because a log axis that reached zero would be a
   * silently blank chart.
   */
  const logScale = (dim: Dimension, fixed: [number, number] | null): boolean =>
    dim.frame === "log" && (fixed?.[0] ?? 0) > 0;
  const xLog = logScale(dx, xFixed);
  const yLog = logScale(dy, yFixed);

  /*
   * How many districts the frame does not hold, counted rather than allowed to set the scale.
   *
   * See `envelope`: the axis is sized to the 99th percentile, so a handful sit beyond it and are
   * clipped. A chart that quietly drops them would be the same defect as one that quietly drops a
   * district with no valuation, so the card says how many and why.
   */
  const outside = points.filter(
    (p) =>
      (xFixed != null && (p.x < xFixed[0] || p.x > xFixed[1])) ||
      (yFixed != null && (p.y < yFixed[0] || p.y > yFixed[1])),
  ).length;

  /*
   * How many districts the highlighted group has, counted from what is actually drawn — and within
   * the scope, because a legend reading "302" beside a cloud of six lit districts is a count of a
   * population the reader is not looking at.
   */
  const lit =
    view.shading === "type"
      ? panel.districts.filter(
          (d) => d.typology?.code === view.highlight && (scope == null || scope.has(d.irn)),
        ).length
      : 0;
  const litLabel =
    panel.districts.find((d) => d.typology?.code === view.highlight)?.typology?.short ?? "";

  /*
   * What the chart is spoken as, which is the one place both counts and the scope meet.
   *
   * Built here rather than inline because it is now four sentences crossed with two modes, and the
   * thing that must not drift is the denominator: `population` is the state under the spotlight and
   * the selection under a subset, so "3 of 6 districts" and "3 of 609" are each said where they are
   * true. See `population`.
   */
  const drawnLabel =
    subset
      ? `${
          unplaced === 0
            ? `All ${count(placed)} districts`
            : `${count(placed)} of the ${count(population)} districts`
        } in ${asked}`
      : unplaced === 0
        ? `Every one of Ohio's ${count(placed)} districts`
        : `${count(placed)} of Ohio's ${count(population)} districts`;
  const scopeClause =
    scope == null
      ? ""
      : subset
        ? ", on the frame the whole state is measured on"
        : `, of which ${count(scoped)} in ${asked} are drawn as the subject and the rest as context`;

  const legend =
    (view.shading === "regime"
      ? REGIMES.map((label, i) => `<span><i class="sw ordinal-${i + 1}"></i> ${label}</span>`).join("")
      : view.shading === "type"
        ? `<span><i class="sw gain"></i> ${escapeHtml(litLabel)} (${count(lit)})</span>
           <span><i class="sw neutral"></i> Every other district</span>`
        : `<span><i class="sw gain"></i> Paid more</span>
           <span><i class="sw loss"></i> Paid less</span>
           <span><i class="sw neutral"></i> Unmoved</span>`) +
    /*
     * The scope's own entry, and the reason `muted` exists.
     *
     * Under the regime and typology colourings an out-of-scope district could have been left
     * neutral and this entry would still have been needed to say so. Under "Gained or lost" neutral
     * already means *unmoved*, and one swatch cannot mean two things — so the channel is opacity
     * and size, and the swatch says which.
     *
     * Gone under a subset, where there is no out-of-scope mark on the chart for it to name. A
     * legend entry for a population that is not drawn is a legend entry a reader looks for.
     */
    (scope == null || subset
      ? ""
      : `<span><i class="sw neutral muted"></i> Outside ${escapeHtml(asked)}, drawn for context</span>`);

  return `
    <div class="card stage" id="positions" data-part="positions">
      <h2>${heading("positions", "Where every district stands", chip)}</h2>
      <div class="chartwrap" data-chart="positions">${renderToString(
        (w) =>
          scatterSpec(
            points,
            {
              x: { label: dx.label, format: dx.format, log: xLog },
              y: { label: dy.label, format: dy.format, log: yLog },
            },
            [],
            /*
             * The identity option shares one domain across both axes and squares the frame, which
             * is only meaningful where the two axes are the same quantity in the same units. On
             * every other pair the frame takes its own height and each axis its own range.
             */
            {
              width: w,
              ...(wall ? { identity: { label: "realized = formula" } } : {}),
              ...(xFixed ? { xDomain: xFixed } : {}),
              ...(yFixed ? { yDomain: yFixed } : {}),
            },
          ),
        {
          label: `${drawnLabel}, ${dx.label.toLowerCase()} against ${dy.label.toLowerCase()}${view.trails ? ", with a trail from its position under current law" : ""}${scopeClause}`,
          description: `${count(scoped - pinned)} ${scope == null ? "districts" : `of the ${count(scoped)} districts in scope`} are paid differently under these settings and ${count(pinned)} are paid the same.${wall ? " No district can fall below the diagonal, where realized aid equals formula aid; a district drawn above it is held by the guarantee and the vertical distance is what the guarantee pays it. A trail that runs flat is a district whose formula amount moved and whose payment did not." : ""}`,
        },
      )}</div>
      <div class="legend">${legend}</div>
      <p class="note">${
        wall
          ? `Nothing can sit below the diagonal: a district receives the larger of its formula
             amount and its guarantee, so the vertical distance above the line is the guarantee
             itself. A held district travels <em>sideways</em> until it reaches the line, and only
             then does a further dollar of base cost reach it. `
          : ""
      }<strong>${count(pinned)} of ${count(scoped)}</strong> ${
        scope == null ? "districts" : `districts in ${escapeHtml(asked)}`
      } ${pinned === 1 ? "is" : "are"} paid
        exactly what current law pays them under these settings.${
          view.trails
            ? " Each district is drawn from where current law puts it to where these levers do, so a trail with no vertical component is a district whose payment did not move."
            : ""
        } This is <em>which</em> districts move;
        <a href="/scenario" data-carry-levers>how much they move</a> is on the scenario runner, at
        the same lever positions.</p>
      ${
        scope == null
          ? ""
          : subset
            ? `<p class="note"><strong>Only ${escapeHtml(asked)} is drawn.</strong> The other ${
                count(panel.statewide.districts - scope.size)
              } districts are not on this chart and not in any count on it. <strong>The frame is
               still the state's</strong>: the axes are measured across every setting these levers
               can reach and over all ${
                 count(panel.statewide.districts)
               } districts, so this selection sits where the state puts it rather than filling a
               ruler fitted to its own points — which is what lets you compare it with another
               county, and with itself at another lever position.${
                 wall
                   ? " The diagonal is the law rather than a fit, so it is drawn for any population: nothing can sit below it here either."
                   : ""
               } <button type="button" class="linkish" data-scope-mode="spotlight">Draw the whole
               state behind it</button> to put the selection back in its context.</p>`
            : `<p class="note"><strong>The other ${count(placed - scoped)} districts are still
             drawn</strong>, muted, and they are not in any count above. A selection here changes
             what is <em>lit</em> rather than what is plotted, for two reasons. The frame is measured
             across every setting these levers can reach and over all ${
               count(panel.statewide.districts)
             } districts, so that a district which did not move looks like one that did not
             move — fitting it to a county instead would put the ruler back on the move. And most
             selections are not a cloud: this form refuses fewer than ${MIN_CLOUD} points, because a
             scatter of three districts reads as a finding about a population nobody measured, and ${
               count(allCounties.length - enough.length)
             } of Ohio's ${count(allCounties.length)} counties hold fewer than ${
               MIN_CLOUD
             } districts. The shape of the state behind your selection is what makes a position in
             it legible.${
               scoped >= MIN_CLOUD
                 ? ` This selection is large enough to stand on its own: <button type="button"
                    class="linkish" data-scope-mode="subset">draw only ${
                      escapeHtml(asked)
                    }</button> to see how its districts are arranged among themselves.`
                 : ""
             }</p>`
      }
      ${
        outside > 0
          ? `<p class="note"><strong>${count(outside)} of ${count(placed)} ${
              subset ? `drawn districts in ${escapeHtml(asked)}` : "districts"
            } ${outside === 1 ? "sits" : "sit"} outside
             the frame.</strong> The axes are fixed across every setting the levers can reach, so
             that a district which did not move looks like one that did not move — and they are
             sized to hold ${pct(FRAME_QUANTILE, 0)} of districts rather than all of them.
             <strong>Kelleys Island Local has four pupils</strong>, and a per-pupil figure on a
             denominator of four is a different kind of quantity: fitting the state's axis to it
             would make every other district a smudge against the left edge. The clipped are still
             in every total on this page.</p>`
          : ""
      }
      ${
        unplaced > 0
          ? `<p class="note"><strong>${count(unplaced)} of ${count(population)} ${
              subset ? `districts in ${escapeHtml(asked)}` : "districts"
            } ${unplaced === 1 ? "is" : "are"} not drawn.</strong> The feed carries no value
             for at least one of the two axes chosen${
               subset
                 ? " — not because you did not select them, which is a different absence and is accounted for above"
                 : ""
             }. Three quantities here can be missing: valuation per pupil and the
             report card's disadvantaged share, which the feed does not always carry, and the
             department's typology, which it declines to assign to five districts — three of them
             in this panel. A district placed at zero for want of a figure would be a point
             asserting something the data does not say, and on the typology axis it would be worse
             than that: zero is the department's refusal, not the rural end of its scale.</p>`
          : ""
      }
    </div>`;
}
