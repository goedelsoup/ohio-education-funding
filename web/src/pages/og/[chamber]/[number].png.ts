/**
 * One card per legislative seat: 99 House, 33 Senate.
 *
 * The card repeats what the page's first paragraph says, because it has to. These figures are
 * apportioned from census blocks and nothing publishes them — no House district is a unit of
 * account in Ohio's funding system — and a preview is exactly the context where a number gets
 * separated from its caveat and quoted. "Apportioned from census blocks" is on the card for the
 * same reason it is on the page.
 */

import type { APIRoute, GetStaticPaths } from "astro";

import { loadFeed } from "../../../lib/feed.ts";
import { count, millions } from "../../../lib/format.ts";
import type { Card } from "../../../lib/og/card.ts";
import { SITE } from "../../../lib/og/pages.ts";
import { respond } from "../../../lib/og/render.ts";
import type { Bundle } from "../../../lib/types.ts";

type Chamber = "house" | "senate";
/** Both chambers are the same shape in the feed, and neither is exported under its own name. */
type Seat = Bundle["house_districts"][number];

export const getStaticPaths: GetStaticPaths = () => {
  const { bundle } = loadFeed();
  return [
    ...bundle.house_districts.map((seat) => ({
      params: { chamber: "house", number: seat.number },
      props: { seat, chamber: "house" as const },
    })),
    ...bundle.senate_districts.map((seat) => ({
      params: { chamber: "senate", number: seat.number },
      props: { seat, chamber: "senate" as const },
    })),
  ];
};

/** Exported for the unit suite. */
export function seatCard(seat: Seat, chamber: Chamber, fiscalYear: number): Card {
  const label = chamber === "house" ? "House" : "Senate";
  const wholly = seat.districts_wholly_inside;
  // Ohio has seats containing exactly one school district, so the plural is not decorative.
  const members = seat.members.length;
  const schools = `${count(members)} school district${members === 1 ? "" : "s"}`;

  return {
    eyebrow: `${SITE} · Ohio ${label}`,
    headline: `${label} District ${Number(seat.number)}`,
    figure: millions(seat.realized_aid).replace("+", ""),
    figureNote: `State aid to the ${schools} in this seat, apportioned from census blocks`,
    meta: `${count(wholly)} wholly inside · ${count(seat.members.length - wholly)} shared with another seat · FY${fiscalYear}`,
  };
}

export const GET: APIRoute = async ({ props }) => {
  const { seat, chamber } = props as { seat: Seat; chamber: Chamber };
  return respond(seatCard(seat, chamber, loadFeed().bundle.fiscal_year));
};
