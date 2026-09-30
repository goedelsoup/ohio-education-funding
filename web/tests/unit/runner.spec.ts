/**
 * The runner's *what changes* view: what it renders per tick, which district a link opens it on,
 * and the address it writes back. The three defects of #563.
 *
 * The browser half — that a list-valued link opens with a note and keeps the list, and that a
 * `#d=` fragment does not outlive a boot — is in `tests/e2e/scenario.spec.ts`.
 */

import { beforeEach, describe, expect, test, vi } from "vitest";

/*
 * `applyAll` is spied rather than timed: the defect is a count, and a count does not flake.
 * Mocked at the module `scenario.ts` imports it from, which is the one binding a spy can reach.
 *
 * And Plot's renderer is stubbed, because the statewide half draws a histogram with it and needs a
 * document this suite deliberately does not have — see `vitest.config.ts`. Nothing here is about
 * the drawing.
 */
vi.mock("../../src/lib/policy.ts", async (original) => {
  const actual = await original<typeof import("../../src/lib/policy.ts")>();
  return { ...actual, applyAll: vi.fn(actual.applyAll) };
});
vi.mock("../../src/lib/plot/client.ts", async (original) => ({
  ...(await original<typeof import("../../src/lib/plot/client.ts")>()),
  renderToString: () => "<svg></svg>",
}));

import { applyAll, modelOf } from "../../src/lib/policy.ts";
import { loadFeed } from "../../src/lib/feed.ts";
import {
  chosenDistrict,
  defaultLevers,
  districtParam,
  namedDistricts,
  nextAddress,
  renderNamed,
  renderRunner,
} from "../../src/lib/scenario.ts";

const CLEVELAND = "043786";
const NORTHERN = "049056";
const FREMONT = "044016";

describe("one tick", () => {
  const { bundle } = loadFeed();
  const moved = { ...defaultLevers(modelOf(bundle.statewide)), baseCostScale: 1.05 };
  const spy = vi.mocked(applyAll);
  // A block, not an arrow's value: `mockClear` returns the mock, and a function returned from
  // `beforeEach` is run as its cleanup — which called `applyAll()` with no arguments.
  beforeEach(() => {
    spy.mockClear();
  });

  test("runs the formula once for a district, not once per half", () => {
    // The script took the district cards and the statewide detail from two renderers, and each ran
    // the whole panel for the same levers — twice per slider frame.
    const rendered = renderRunner(bundle, moved, CLEVELAND);
    expect(spy).toHaveBeenCalledTimes(1);
    // And both halves are still there, or one call would be easy.
    expect(rendered.summary).toContain('data-part="outcome"');
    expect(rendered.summary).toContain("State aid under this scenario");
    expect(rendered.detail).toContain('data-part="distribution"');
    expect(rendered.detail).toContain('data-part="moved-underneath"');
  });

  test("runs it once for the state", () => {
    const rendered = renderRunner(bundle, moved, "");
    expect(spy).toHaveBeenCalledTimes(1);
    expect(rendered.summary).toContain("What these levers do");
  });

  test("the shared run is the run: a district's figures match the district rendered alone", () => {
    const alone = renderRunner(bundle, moved, NORTHERN).summary;
    renderRunner(bundle, moved, CLEVELAND);
    expect(renderRunner(bundle, moved, NORTHERN).summary).toBe(alone);
  });
});

describe("a link naming several districts", () => {
  const named = (search: string, hash = "") => namedDistricts(new URLSearchParams(search), hash);

  test("keeps the whole list, in order, once each", () => {
    expect(named(`?d=${NORTHERN},${CLEVELAND},${NORTHERN}`)).toEqual([NORTHERN, CLEVELAND]);
    expect(named(`?d=${NORTHERN},cleveland,${CLEVELAND}`)).toEqual([NORTHERN, CLEVELAND]);
    // And the view still opens on the first.
    expect(chosenDistrict(new URLSearchParams(`?d=${NORTHERN},${CLEVELAND}`), "")).toBe(NORTHERN);
  });

  test("a redirect's fragment names one, and wins", () => {
    expect(named(`?d=${NORTHERN},${CLEVELAND}`, `#d=${FREMONT}`)).toEqual([FREMONT]);
  });

  test("is written back behind the district chosen, while it is one of them", () => {
    const list = [NORTHERN, CLEVELAND, FREMONT];
    // What the old write-back did was `d=<first>`, which dropped the rest on the first tick.
    expect(districtParam(NORTHERN, list)).toBe(`${NORTHERN},${CLEVELAND},${FREMONT}`);
    // Chosen first, so the same district opens off a copied URL.
    expect(districtParam(CLEVELAND, list)).toBe(`${CLEVELAND},${NORTHERN},${FREMONT}`);
    // A district the link did not name, or the state, leaves the list.
    expect(districtParam("000442", list)).toBe("000442");
    expect(districtParam("", list)).toBe("");
    expect(districtParam(NORTHERN, [NORTHERN])).toBe(NORTHERN);
  });

  test("is said, with the others offered", () => {
    const nameOf = (irn: string) => ({ [NORTHERN]: "Northern", [CLEVELAND]: "Cleveland", [FREMONT]: "Fremont & Co" })[irn] ?? irn;
    const note = renderNamed([NORTHERN, CLEVELAND, FREMONT], NORTHERN, nameOf);
    expect(note).toContain("named 3 districts");
    expect(note).toContain("this is Northern");
    expect(note).toContain(`data-named-district="${CLEVELAND}"`);
    expect(note).toContain(`data-named-district="${FREMONT}"`);
    expect(note).not.toContain(`data-named-district="${NORTHERN}"`);
    expect(note).toContain("Fremont &amp; Co");
    expect(note).toContain("data-carry-levers");
    // Nothing to say for one district, or once the reader has left the list.
    expect(renderNamed([NORTHERN], NORTHERN, nameOf)).toBe("");
    expect(renderNamed([NORTHERN, CLEVELAND], "000442", nameOf)).toBe("");
    expect(renderNamed([NORTHERN, CLEVELAND], "", nameOf)).toBe("");
  });
});

describe("the address written back", () => {
  const at = (pathname: string, search: string, hash: string) => ({ pathname, search, hash });

  test("drops a redirect's #d= even when the query already reads right", () => {
    // The old write compared path and query alone, found them equal, and wrote nothing — leaving a
    // fragment that `chosenDistrict` prefers over the query.
    const query = `g=removed&d=${CLEVELAND}`;
    expect(nextAddress(at("/scenario", `?${query}`, `#d=${NORTHERN}`), query)).toBe(
      `/scenario?${query}`,
    );
  });

  test("writes nothing when nothing changes", () => {
    expect(nextAddress(at("/scenario", "?g=removed", ""), "g=removed")).toBeNull();
    expect(nextAddress(at("/scenario", "?g=removed", "#distribution"), "g=removed")).toBeNull();
  });

  test("keeps any other fragment", () => {
    expect(nextAddress(at("/scenario", "?g=removed", "#distribution"), "g=rebase")).toBe(
      "/scenario?g=rebase#distribution",
    );
  });
});
