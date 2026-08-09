/**
 * The apportionment, checked against the one thing it guarantees.
 *
 * Every House district figure on this site is derived — no House district is a unit of account
 * anywhere in Ohio's funding system. The single accuracy claim the derivation can make is that it
 * conserves: each school district's shares sum to one, so the 99 House districts sum to the
 * statewide total. If that failed, money would be created or destroyed by the act of dividing it
 * and a reader adding the rows would find a discrepancy with nothing on the page to explain it.
 */

import { expect, test } from "vitest";

import { loadFeed } from "../../src/lib/feed.ts";

const { bundle } = loadFeed();
const house = bundle.house_districts;

test("Ohio's ninety-nine House districts are all present and all contain schools", () => {
  expect(house).toHaveLength(99);
  expect(house.every((h) => h.members.length > 0)).toBe(true);
  const numbers = house.map((h) => h.number).sort();
  expect(numbers[0]).toBe("001");
  expect(numbers[numbers.length - 1]).toBe("099");
  expect(new Set(numbers).size).toBe(99);
});

test("the apportionment conserves: House districts sum to the statewide totals", () => {
  const districtTotal = (pick: (d: (typeof bundle.districts)[number]) => number) =>
    bundle.districts.reduce((a, d) => a + pick(d), 0);

  for (const [label, apportioned, exact] of [
    ["state aid", house.reduce((a, h) => a + h.realized_aid, 0), districtTotal((d) => d.realized_aid_per_pupil * d.adm)],
    ["guarantee", house.reduce((a, h) => a + h.guarantee, 0), districtTotal((d) => d.guarantee)],
    ["base cost share", house.reduce((a, h) => a + h.base_cost_state_share, 0), districtTotal((d) => d.base_cost_state_share)],
    ["categoricals", house.reduce((a, h) => a + h.categorical_funding, 0), districtTotal((d) => d.categorical_funding)],
    ["pupils", house.reduce((a, h) => a + h.adm, 0), districtTotal((d) => d.adm)],
  ] as const) {
    expect(Math.abs(apportioned - exact) / Math.max(Math.abs(exact), 1), label).toBeLessThan(1e-8);
  }
});

test("every district's shares sum to one, and every district has at least one seat", () => {
  for (const d of bundle.districts) {
    expect(d.house_districts.length, d.name).toBeGreaterThan(0);
    const total = d.house_districts.reduce((a, h) => a + h.share, 0);
    expect(Math.abs(total - 1), d.name).toBeLessThan(1e-6);
  }
});

test("a member's two shares mean different things and neither is the other", () => {
  /*
   * `share` is how much of the school district is in this House district; `share_of_house_district`
   * is how much of this House district's pupils that school district provides. The page prints both
   * side by side, so a swap would be invisible — 100% in the wrong column reads as "the member
   * speaks for all of it" when it means "this seat is entirely that district".
   */
  for (const h of house) {
    const ofSeat = h.members.reduce((a, m) => a + m.share_of_house_district, 0);
    expect(Math.abs(ofSeat - 1), `House ${h.number}`).toBeLessThan(1e-6);
  }
  // And they genuinely differ somewhere, or the test above would pass on a duplicated field.
  const differing = house
    .flatMap((h) => h.members)
    .filter((m) => Math.abs(m.share - m.share_of_house_district) > 0.01);
  expect(differing.length).toBeGreaterThan(100);
});

test("wholly-inside is consistent with the district's own share list", () => {
  const spans = new Map(bundle.districts.map((d) => [d.irn, d.house_districts.length]));
  for (const h of house) {
    for (const m of h.members) {
      expect(m.wholly_inside, `${m.name} in House ${h.number}`).toBe(spans.get(m.irn) === 1);
      if (m.wholly_inside) expect(Math.abs(m.share - 1)).toBeLessThan(1e-6);
    }
  }
});

test("most districts straddle a House district boundary, which is why this is apportioned", () => {
  // The fact that rules out assigning each district to one seat, as the county pages do. If it
  // stopped being true the simpler design would be available and the framing would need revisiting.
  const split = bundle.districts.filter((d) => d.house_districts.length > 1).length;
  expect(split).toBeGreaterThan(bundle.districts.length / 2);
  expect(Math.max(...bundle.districts.map((d) => d.house_districts.length))).toBeGreaterThanOrEqual(10);
});
