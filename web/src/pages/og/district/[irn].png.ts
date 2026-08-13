/**
 * One card per district. 609 of them, and the largest single block of work in the build.
 *
 * # One card, five pages
 *
 * A district has five routes — the dashboard, finances, outcome, scenario and taxes — and they all
 * point at this one image. What separates them in a feed is `og:title`, which carries each page's
 * own title, so a share of `/district/043786/taxes` reads "Cleveland Municipal — property tax"
 * above a card showing Cleveland's aid. Five renderings per district would be 3,045 images for a
 * difference no reader would see, at five times the build cost and five times the deploy.
 *
 * # What the card says
 *
 * State aid per pupil, and where it comes from. That is the question the site exists to answer and
 * the one a reader can act on before opening anything: a district on the guarantee is being paid
 * by a mechanism the legislature describes as temporary, and the bar says how much of its aid that
 * mechanism accounts for.
 */

import type { APIRoute } from "astro";

import { districtPaths, loadFeed } from "../../../lib/feed.ts";
import { count, money, pct } from "../../../lib/format.ts";
import type { Card } from "../../../lib/og/card.ts";
import { SITE } from "../../../lib/og/pages.ts";
import { respond } from "../../../lib/og/render.ts";
import type { District } from "../../../lib/types.ts";

/** The same 609 paths the district pages are built from, so the two sets cannot diverge. */
export const getStaticPaths = districtPaths;

/** Exported for the unit suite, which renders the extremes rather than trusting the layout. */
export function districtCard(district: District, fiscalYear: number): Card {
  const guaranteePP = district.realized_aid_per_pupil - district.formula_aid_per_pupil;
  /*
   * The share of *aid* the guarantee accounts for, not the share of districts on it.
   *
   * Clamped rather than trusted: `formula_aid_per_pupil` exceeds `realized_aid_per_pupil` for a
   * district whose formula amount is reduced on the way to what it is paid, which would make this
   * negative. The bar renders 0 there, which is the honest reading — no part of what this district
   * receives is guarantee money.
   */
  const guaranteeShare =
    district.realized_aid_per_pupil > 0 ? guaranteePP / district.realized_aid_per_pupil : 0;

  return {
    eyebrow: `${SITE} · ${district.county} County`,
    headline: district.name,
    figure: `${money(district.realized_aid_per_pupil)} per pupil`,
    figureNote: district.on_guarantee
      ? `In state aid, of which ${money(guaranteePP)} comes from the transitional guarantee rather than the formula`
      : "In state aid, all of it computed by the formula",
    bar: district.on_guarantee
      ? {
          share: guaranteeShare,
          label: `${pct(guaranteeShare, 0)} from the guarantee`,
          tone: "guarantee",
        }
      : { share: 1, label: "Funded by the formula", tone: "formula" },
    meta: `IRN ${district.irn} · ${count(Math.round(district.adm))} pupils · FY${fiscalYear}`,
  };
}

export const GET: APIRoute = async ({ props }) =>
  respond(districtCard((props as { district: District }).district, loadFeed().bundle.fiscal_year));
