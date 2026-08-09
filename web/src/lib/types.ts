/**
 * The feed's types, for everything that reads it.
 *
 * The shapes themselves are declared once, as zod schemas, in `schema/feed.ts` — so the build can
 * *parse* the feed rather than cast it, and a Rust field that changes name stops the build instead
 * of rendering as an em dash on 609 pages.
 *
 * This module exists so that the browser never sees any of that. Every re-export below is
 * `export type`, which TypeScript erases: client code importing `Panel` from here does not pull
 * zod, or the schemas, or anything else, into the bundle. `REQUIRED_CONTRACT` is the one value,
 * and it is a string.
 *
 * Import types from here. Import schemas from `schema/feed.ts`, and only in build-time code.
 */

export type {
  BaseCostBuildUp,
  Bundle,
  MillageAnalysis,
  National,
  StateFinance,
  RegimeCounterfactual,
  PropertyTaxYear,
  SpendingByFunction,
  Checkpoint,
  Deflator,
  District,
  DistrictOutcome,
  FinanceYear,
  ForecastCheckpoint,
  OutcomeStatewide,
  PolicyShape,
  ProjectionMeta,
  Statewide,
} from "./schema/feed.ts";

import type { Bundle, District, Statewide } from "./schema/feed.ts";

/**
 * The bundle contract this site understands.
 *
 * `CONTRACT_VERSION` in `crates/bundle` is bumped whenever a field changes meaning. This is what
 * the build and the scenario routes refuse to proceed past when the two disagree — the deliberate
 * half of drift detection, where the strictness of the schemas is the accidental half.
 */
export const REQUIRED_CONTRACT = "18.0.0";

/**
 * A district with only the fields the funding formula reads.
 *
 * # What this type is for
 *
 * The scenario routes re-run the formula in the browser, so they need the whole 609-district panel
 * rather than one district's page. Sending the full feed for that costs 202 KB gzipped, and most
 * of it — six years of audited finances and a report card per district — is untouched by
 * `policy.ts` and `project.ts`. Dropping those blocks costs 68 KB instead.
 *
 * `base_cost_build_up` is dropped for the same reason and is the largest of the three: twenty-nine
 * numbers per district, half a megabyte across the panel, and the formula reads none of them. It
 * explains a figure the scenario routes only ever consume as a total.
 *
 * `property_tax`, `spending_by_function` and `millage` go the same way. None is an input to the
 * funding formula — one is the local tax base the state charges against, one is what a district
 * did with the money afterwards, and the third is a reading of H.B. 920 that the scenario builder
 * deliberately does not model — so the compiler keeps all three out of the browser's copy.
 *
 * The six categorical decompositions go out too, and they are the largest addition the feed has
 * taken: forty-one numbers per district across special education, career-technical, English
 * learners, DPIA, targeted assistance and gifted. They exist so a district's page can say *how* its
 * categorical money was computed, and the scenario builder consumes only the six totals in
 * `categoricals`. Shipping the mechanism to a browser that re-runs the formula from the totals
 * would be a quarter of a megabyte for nothing.
 *
 * The scenario holds valuation fixed, and `millage` is the reason it must: reduction factors make
 * a district's response to a reappraisal depend on which side of the floor it is on, and the feed
 * has two observations per district. Shipping the block would invite a slider that cannot honestly
 * exist yet.
 *
 * Making that a *type* rather than a comment is the point. Every function in the formula takes a
 * `PanelDistrict`, so "does the browser need this field?" is answered by the compiler: reach for
 * `finances` inside `apply()` and the build fails rather than the slim panel silently arriving
 * without it.
 *
 * A full {@link District} is assignable to this, so the build-time paths that do have the whole
 * feed pass it unchanged.
 */
export type PanelDistrict = Omit<
  District,
  | "finances"
  | "outcome"
  | "base_cost_build_up"
  | "property_tax"
  | "spending_by_function"
  | "millage"
  | "regime"
  | "special_education"
  | "career_technical"
  | "english_learners"
  | "dpia"
  | "targeted_assistance"
  | "gifted"
>;

/** The feed with the two heavy per-district blocks removed. Served as `/data/panel.json`. */
export interface Panel extends Omit<Bundle, "districts" | "statewide" | "national"> {
  districts: PanelDistrict[];
  statewide: Omit<Statewide, "finances" | "outcomes">;
}
