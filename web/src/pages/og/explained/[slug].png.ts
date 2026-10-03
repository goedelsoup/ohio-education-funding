/**
 * One card per Explained topic, under the section as the page is (#713).
 *
 * The cards are `explainedCards()` in `src/lib/og/pages.ts`, built from the topic registry, so a
 * topic module gets its card the day it is registered.
 */

import type { APIRoute, GetStaticPaths } from "astro";

import { explainedCards } from "../../../lib/og/pages.ts";
import { respond } from "../../../lib/og/render.ts";
import type { Card } from "../../../lib/og/card.ts";

export const getStaticPaths: GetStaticPaths = () =>
  Object.entries(explainedCards()).map(([slug, card]) => ({ params: { slug }, props: { card } }));

export const GET: APIRoute = async ({ props }) => respond((props as { card: Card }).card);
