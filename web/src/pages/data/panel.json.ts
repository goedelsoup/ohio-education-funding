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

import { formulaInputs, loadFeed } from "../../lib/feed.ts";
import type { Panel } from "../../lib/types.ts";

export const GET: APIRoute = () => {
  const { bundle } = loadFeed();
  const { finances, outcomes, ...statewide } = bundle.statewide;

  const panel: Panel = {
    contract_version: bundle.contract_version,
    provenance: bundle.provenance,
    fiscal_year: bundle.fiscal_year,
    // Ten short rows, and the scenario runner is the one surface that renders figures the reader
    // never saw the build-time page for. Trimming this to save a few hundred bytes would be
    // trimming the only thing that says what year the numbers beside it are.
    series_years: bundle.series_years,
    statewide,
    checkpoints: bundle.checkpoints,
    // The drafts travel with the panel because the scenario page is where a draft is opened, and
    // a draft opened without its unpriced provisions is the one thing this whole class refuses.
    // Six short rows.
    drafts: bundle.drafts,
    projection: bundle.projection,
    // The Census comparison is not a formula input, so it stays out of the browser's copy.
    deflator: bundle.deflator,
    districts: bundle.districts.map(formulaInputs),
  };

  return new Response(JSON.stringify(panel), {
    headers: { "content-type": "application/json; charset=utf-8" },
  });
};
