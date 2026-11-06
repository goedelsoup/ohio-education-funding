/**
 * The district card for chartered nonpublic schools (#765), and the two things it must not say.
 *
 * Auxiliary services is paid through the district a school is located in, so the card invites a
 * dollar figure. Nothing this site reads says what a district received, so the card carries none.
 * And a masked count is a range, so the card never prints one end of it as the count.
 */

import { experimental_AstroContainer as AstroContainer } from "astro/container";
import { parseHTML } from "linkedom";
import { expect, test } from "vitest";

import Nonpublic from "../../src/components/district/Nonpublic.astro";
import { loadFeed } from "../../src/lib/feed.ts";
import type { District } from "../../src/lib/types.ts";

const { bundle } = loadFeed();
const container = await AstroContainer.create();
const render = async (district: District) =>
  parseHTML(`<div>${await container.renderToString(Nonpublic, { props: { bundle, district } })}</div>`).document;

const byIrn = (irn: string) => bundle.districts.find((d) => d.irn === irn)!;

test("the district lists sum to the placement count the statewide card quotes", () => {
  const placement = bundle.funding_units!.nonpublic_support.placement;
  const listed = bundle.districts.reduce((sum, d) => sum + d.nonpublic_schools.length, 0);
  expect(listed).toBe(placement.listed);
  const irns = bundle.districts.flatMap((d) => d.nonpublic_schools.map((s) => s.irn));
  expect(new Set(irns).size, "no school on two district pages").toBe(irns.length);
});

test("a masked school prints its range and an exact one its count", async () => {
  const cincinnati = byIrn("043752");
  expect(cincinnati.nonpublic_schools.length).toBeGreaterThan(12);
  const document = await render(cincinnati);
  const cells = [...document.querySelectorAll("td.tnum")].map((td) => td.textContent!.trim());
  expect(cells).toHaveLength(cincinnati.nonpublic_schools.length);
  cincinnati.nonpublic_schools.forEach((school, i) => {
    expect(cells[i]!.includes("–"), school.name).toBe(school.floor !== school.ceiling);
  });
  // Past twelve, the rest are folded rather than dropped.
  expect(document.querySelector("details.table-rest")).not.toBeNull();
});

test("no district card carries a dollar figure", async () => {
  for (const irn of ["043752", "043802", "043786"]) {
    const text = (await render(byIrn(irn))).body?.textContent ?? "";
    expect(text, irn).not.toMatch(/\$\d/);
  }
});

test("a district with no school placed says so, and says why that is not none", async () => {
  const none = bundle.districts.find((d) => d.nonpublic_schools.length === 0)!;
  const document = await render(none);
  expect(document.querySelector("table")).toBeNull();
  expect(document.querySelector("#nonpublic")!.textContent).toMatch(/could not be placed/);
});
