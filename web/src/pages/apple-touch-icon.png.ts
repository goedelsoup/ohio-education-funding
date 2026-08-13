/**
 * The home-screen icon, rasterized from `public/favicon.svg`.
 *
 * 180×180 is what iOS asks for and what it scales down from for every smaller slot it needs.
 *
 * Deliberately not transparent: iOS composites a home-screen icon onto whatever the wallpaper is,
 * and the mark's own surface is what keeps it legible there. The rounded corners are drawn in the
 * SVG, so the raster keeps them and the platform's own mask has nothing to fight with.
 */

import type { APIRoute } from "astro";

import { mark } from "../lib/og/mark.ts";
import { rasterize } from "../lib/og/render.ts";

export const GET: APIRoute = () =>
  new Response(new Uint8Array(rasterize(mark(), 180)), {
    headers: { "Content-Type": "image/png" },
  });
