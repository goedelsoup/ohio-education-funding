/**
 * The PNG fallback for `rel="icon"`, rasterized from `public/favicon.svg`.
 *
 * Every current browser takes `image/svg+xml` for a favicon, so this is not for browsers. It is for
 * the consumers that will not — some feed readers, and several of the link unfurlers that render a
 * site icon beside a preview card. Those are exactly the clients this change exists to serve, so
 * shipping only the SVG would leave the icon missing in the one place it was added for.
 */

import type { APIRoute } from "astro";

import { mark } from "../lib/og/mark.ts";
import { rasterize } from "../lib/og/render.ts";

export const GET: APIRoute = () =>
  new Response(new Uint8Array(rasterize(mark(), 32)), {
    headers: { "Content-Type": "image/png" },
  });
