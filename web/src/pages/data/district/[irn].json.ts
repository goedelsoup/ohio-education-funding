/**
 * One district's formula inputs, for the two `/compare` is about.
 *
 * # The measurement this exists for
 *
 * `/compare` used to download the whole 609-district panel — 641,042 B, 127,961 gzipped — to
 * render seventeen rows about two districts, and showed an empty box until it landed. At
 * 400 Kbps / 400 ms RTT that was first paint at 1,284 ms and a table at 4,340 ms. One district is
 * about a kilobyte, so the pair is two of these instead, and the *default* pair is in the document
 * before any of it: see `compare.astro`. That is the last open half of #111.
 *
 * # Why 609 files rather than one slim index
 *
 * An index of the seventeen fields the table reads, over every district, is 42,607 B gzipped — a
 * threefold cut, and still three orders of magnitude more than the two districts a reader asked
 * for. The picker needs no data at all: `compare.astro` renders all 609 names into its `<select>`
 * at build time, and a name is the only thing the browser wants about a district it is not
 * showing. So the split is per district, and a swap costs about a kilobyte.
 *
 * Same shape and same stripping as `/data/panel.json`, through `formulaInputs`, so the two cannot
 * describe a district differently.
 */

import type { APIRoute, GetStaticPaths } from "astro";

import { formulaInputs, loadFeed } from "../../../lib/feed.ts";

export const getStaticPaths: GetStaticPaths = () =>
  loadFeed().bundle.districts.map((district) => ({ params: { irn: district.irn } }));

export const GET: APIRoute = ({ params }) => {
  const district = loadFeed().bundle.districts.find((d) => d.irn === params.irn);
  if (!district) return new Response("Not found", { status: 404 });
  return new Response(JSON.stringify(formulaInputs(district)), {
    headers: { "content-type": "application/json; charset=utf-8" },
  });
};
