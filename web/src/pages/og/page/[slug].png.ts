/**
 * The cards for the pages that are written rather than derived.
 *
 * The table is `src/lib/og/pages.ts`; this file only turns its keys into routes. A slug that is
 * referenced from a page but absent from the table therefore fails as a missing file in `dist/`,
 * which the artefact scan in `tests/e2e/app.spec.ts` catches — rather than as a 404 nobody sees
 * until a preview comes back blank.
 */

import type { APIRoute, GetStaticPaths } from "astro";

import { pageCards } from "../../../lib/og/pages.ts";
import { respond } from "../../../lib/og/render.ts";
import type { Card } from "../../../lib/og/card.ts";

export const getStaticPaths: GetStaticPaths = () =>
  Object.entries(pageCards()).map(([slug, card]) => ({ params: { slug }, props: { card } }));

export const GET: APIRoute = async ({ props }) => respond((props as { card: Card }).card);
