/**
 * One card per county, 88 of them.
 *
 * The figure is the internal disparity — richest district over poorest on assessed valuation per
 * pupil — because that is what a county page is for. A county is not a unit of account in Ohio's
 * funding system; it is a place where districts with very different tax bases sit next to each
 * other, and the ratio is the whole reason to group them.
 *
 * Four counties have fewer than two reporting districts and so have no ratio. They get the aid
 * figure instead, which is a different card rather than a card with a gap in it.
 */

import type { APIRoute, GetStaticPaths } from "astro";

import { counties, type County } from "../../../lib/county.ts";
import { loadFeed } from "../../../lib/feed.ts";
import { count } from "../../../lib/format.ts";
import type { Card } from "../../../lib/og/card.ts";
import { SITE } from "../../../lib/og/pages.ts";
import { respond } from "../../../lib/og/render.ts";

export const getStaticPaths: GetStaticPaths = () =>
  counties(loadFeed().bundle.districts).map((county) => ({
    params: { slug: county.slug },
    props: { county },
  }));

/** Exported so the unit suite can render the one-district counties without a build. */
export function countyCard(county: County, fiscalYear: number): Card {
  const districts = `${count(county.districts.length)} district${county.districts.length === 1 ? "" : "s"}`;
  const guaranteeShare = county.districts.length
    ? county.onGuarantee / county.districts.length
    : 0;

  return {
    eyebrow: `${SITE} · Counties`,
    headline: `${county.name} County`,
    figure:
      county.valuationRatio == null
        ? districts
        : `${county.valuationRatio.toFixed(1)}× property wealth gap`,
    figureNote:
      county.valuationRatio == null
        ? // A ratio needs two districts. Saying so is a real answer, and better than a blank.
          "One reporting district, so there is no gap within the county to measure"
        : `Between the richest and poorest of its ${districts}, per pupil`,
    // Spread rather than `bar: … : undefined`, which `exactOptionalPropertyTypes` rejects — and
    // rightly: "no bar" and "a bar whose value is undefined" are different claims about the card.
    ...(county.onGuarantee > 0
      ? {
          bar: {
            share: guaranteeShare,
            label: `${count(county.onGuarantee)} of ${districts} on the guarantee`,
            tone: "guarantee" as const,
          },
        }
      : {}),
    meta: `${count(Math.round(county.adm))} pupils · FY${fiscalYear}`,
  };
}

export const GET: APIRoute = async ({ props }) =>
  respond(countyCard((props as { county: County }).county, loadFeed().bundle.fiscal_year));
