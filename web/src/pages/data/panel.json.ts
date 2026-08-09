/**
 * The feed, minus everything the funding formula does not read.
 *
 * The scenario routes re-run Ohio's formula in the browser, so unlike every other page they need
 * the whole 609-district panel rather than one district's figures. The full feed costs 202 KB
 * gzipped and most of that — six closed years of audited finances and a report card for each
 * district — is untouched by `policy.ts` and `project.ts`. Dropping those two blocks brings it to
 * 68 KB.
 *
 * What may be dropped is not a judgement call here: {@link PanelDistrict} is the type the whole
 * formula is written against, so the compiler rejects any use of a field this endpoint omits. If
 * a future lever needs report-card data, `apply()` stops compiling before this file is wrong.
 *
 * The complete feed is still published at `/data/bundle.json` and is what `/data` offers for
 * download. This is a transport optimisation, not a different dataset.
 */

import type { APIRoute } from "astro";

import { loadFeed } from "../../lib/feed.ts";
import type { Panel } from "../../lib/types.ts";

export const GET: APIRoute = () => {
  const { bundle } = loadFeed();
  const { finances, outcomes, ...statewide } = bundle.statewide;

  const panel: Panel = {
    contract_version: bundle.contract_version,
    provenance: bundle.provenance,
    fiscal_year: bundle.fiscal_year,
    statewide,
    checkpoints: bundle.checkpoints,
    projection: bundle.projection,
    // The Census comparison is not a formula input, so it stays out of the browser's copy.
    deflator: bundle.deflator,
    districts: bundle.districts.map(
      ({
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
        ...district
      }) => district,
    ),
  };

  return new Response(JSON.stringify(panel), {
    headers: { "content-type": "application/json; charset=utf-8" },
  });
};
