/**
 * The card a page gets when it has not been given one.
 *
 * `Base.astro` defaults to this, so a route added later previews as the site rather than as
 * nothing — the failure mode worth designing for, since a missing `og:image` is invisible until
 * someone pastes the link somewhere.
 *
 * It renders `pageCards().site`, which is also what `/og/page/site.png` would render. That is one
 * image in two places rather than two images; the duplicate costs 26 KB and buys a stable,
 * memorable path for the layout's default that does not depend on a table key.
 */

import type { APIRoute } from "astro";

import { pageCards } from "../../lib/og/pages.ts";
import { respond } from "../../lib/og/render.ts";

export const GET: APIRoute = async () => respond(pageCards().site!);
