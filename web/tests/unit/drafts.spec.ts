/**
 * A draft cannot reach the page without what it could not price.
 *
 * `crates/project::drafts::Priced` has no constructor that skips the unpriced provisions, so the
 * CLI cannot print a bill's cost without them. That guarantee stops at the process boundary: the
 * feed is a JSON file and a page reading only the lever positions out of it would show a statewide
 * total for two of a bill's five clauses with nothing on screen saying so.
 *
 * These are the checks that carry the rule across that boundary.
 */

import { expect, test } from "vitest";

import { loadFeed } from "../../src/lib/feed.ts";
import {
  defaultLevers,
  draftLevers,
  matchesDraft,
  renderDraft,
} from "../../src/lib/scenario.ts";
import { applyAll, totals } from "../../src/lib/policy.ts";
import { toPolicy } from "../../src/lib/scenario.ts";
import type { Panel } from "../../src/lib/types.ts";

const { bundle } = loadFeed();

/** The feed as the scenario page receives it, which is the panel rather than the full bundle. */
function panel(): Panel {
  const { finances: _f, outcomes: _o, ...statewide } = bundle.statewide;
  return { ...bundle, statewide, districts: bundle.districts } as unknown as Panel;
}

/**
 * Collapse whitespace before matching a phrase.
 *
 * The templates wrap, so `no longer match the\n        draft` is one phrase to a reader and two
 * to `String.contains`. HTML collapses it on the way to the screen; this collapses it on the way
 * to the assertion, so a test is checking what is rendered rather than where a line broke.
 */
const flat = (html: string): string => html.replace(/\s+/g, " ");

const MODEL = bundle.statewide.minimum_state_share;
const BASE_YEAR = bundle.projection?.base_year ?? 0;

test("the feed carries every draft, unpriced provisions included", () => {
  // The filter that would break this is the tempting one: exporting only the provisions that set a
  // lever, because those are the ones the runner uses. What the runner uses and what the reader
  // has to be told are different sets, and the second is larger.
  expect(bundle.drafts.length).toBeGreaterThan(0);
  const withUnpriced = bundle.drafts.filter((d) =>
    d.provisions.some((p) => p.lever === ""),
  );
  expect(
    withUnpriced.length,
    "at least one draft must carry a provision no lever reaches, or this rule is untested",
  ).toBeGreaterThan(0);
});

test("every unpriced provision says what it would need", () => {
  // "Not runnable" alone is a shrug. Enforced in Rust too; asserted again here because the feed is
  // what the page reads and a serializer that dropped the field would pass the Rust test.
  for (const draft of bundle.drafts) {
    for (const provision of draft.provisions.filter((p) => p.lever === "")) {
      expect(provision.note.length, `${draft.slug} #${provision.ordinal}`).toBeGreaterThan(40);
    }
  }
});

test("a draft's levers reproduce the policy its provisions describe", () => {
  // The web's `draftLevers` is a third implementation of the binding — after the fixture and the
  // Rust — so it gets the same treatment `policy.ts` gets: it has to agree with a figure computed
  // by Rust rather than be trusted.
  const draft = bundle.drafts.find((d) => d.slug === "hb-96-with-refreshed-inputs");
  expect(draft).toBeDefined();
  const levers = draftLevers(draft!, MODEL, BASE_YEAR);
  expect(levers.baseCostScale).toBeCloseTo(1.0395, 6);
  expect(levers.guarantee).toBe("as-enacted");

  const outcomes = applyAll(bundle.districts, toPolicy(levers), MODEL);
  const t = totals(outcomes);
  // The same $220.5M `crates/project` prints for `--draft hb-96-with-refreshed-inputs`, and the
  // same figure `scenario-delta` pins at 356 gainers.
  expect(t.cost / 1e6).toBeCloseTo(220.5, 0);
  expect(t.gainers).toBe(356);
});

test("a multi-provision draft sets every lever its provisions name", () => {
  const draft = bundle.drafts.find((d) => d.slug === "fund-the-plan-and-retire-the-guarantee");
  expect(draft).toBeDefined();
  const levers = draftLevers(draft!, MODEL, BASE_YEAR);
  expect(levers.baseCostScale).toBeCloseTo(1.0395, 6);
  expect(levers.guarantee).toBe("phase-out");
  expect(levers.guaranteeArgument).toBeCloseTo(0.5, 6);

  const t = totals(applyAll(bundle.districts, toPolicy(levers), MODEL));
  // -$143.9M, and the two provisions priced apart say -$219.0M. The web arrives at the combined
  // figure because it applies both levers to one policy, which is the only arrangement that can.
  expect(t.cost / 1e6).toBeCloseTo(-143.9, 0);
  expect(t.unmoved).toBe(0);
});

test("the rendered card names every provision the model cannot price", () => {
  const draft = bundle.drafts.find((d) => d.slug === "fund-the-plan-and-retire-the-guarantee")!;
  const html = renderDraft(panel(), draftLevers(draft, MODEL, BASE_YEAR), draft.slug);
  for (const provision of draft.provisions.filter((p) => p.lever === "")) {
    expect(html, `provision ${provision.ordinal} is missing from the card`).toContain(
      provision.title,
    );
  }
  expect(flat(html)).toContain("not in any figure on this page");
});

test("a draft that prices completely says so rather than staying silent", () => {
  // Silence would read as "nothing was left out", which is true here and will not be true of the
  // next draft. The card states the ratio in both cases so a reader learns to look for it.
  const draft = bundle.drafts.find((d) => d.slug === "hb-96-with-refreshed-inputs")!;
  const html = renderDraft(panel(), draftLevers(draft, MODEL, BASE_YEAR), draft.slug);
  expect(html).toContain("draft-complete");
  expect(flat(html)).toContain("a property of a one-clause draft");
});

test("moving a lever off the draft is reported, not silently accepted", () => {
  /*
   * The failure this prevents is subtle and likely. A reader opens a bill, nudges a slider to see
   * what happens, and the page still says "opened from a draft" above a number that is no longer
   * the bill's. Removing the banner on the first change would be worse — they would be left with a
   * figure they still believe is the bill's and nothing to correct them.
   */
  const draft = bundle.drafts.find((d) => d.slug === "fund-the-plan-and-retire-the-guarantee")!;
  const moved = { ...draftLevers(draft, MODEL, BASE_YEAR), baseCostScale: 1.08 };

  expect(matchesDraft(moved, draft, MODEL, BASE_YEAR)).toBe(false);
  const html = renderDraft(panel(), moved, draft.slug);
  expect(html).toContain("draft-departed");
  expect(flat(html)).toContain("no longer match the draft");

  // And the unpriced provisions stay on screen. A departed scenario is still missing the same
  // clauses, so dropping them with the banner would trade one wrong impression for another.
  expect(html).toContain("draft-unpriced");
});

test("the horizon is not part of matching a draft", () => {
  // Projecting further out changes what is being asked, not what the bill would do — the same
  // exclusion `isCurrentLaw` makes for the same reason.
  const draft = bundle.drafts.find((d) => d.slug === "hb-96-with-refreshed-inputs")!;
  const levers = draftLevers(draft, MODEL, BASE_YEAR);
  expect(matchesDraft({ ...levers, horizon: levers.horizon + 3 }, draft, MODEL, BASE_YEAR)).toBe(
    true,
  );
});

test("an unknown draft renders nothing rather than an empty card", () => {
  // A `?draft=` naming something that does not exist is a stale link, and a card headed "opened
  // from a draft" with no draft in it would be worse than no card.
  expect(renderDraft(panel(), defaultLevers(MODEL, BASE_YEAR), "no-such-bill")).toBe("");
});

test("a draft nothing prices says the page is not the bill", () => {
  /*
   * The web half of the defect H.B. 643 found. A draft whose provisions all fall outside the
   * model sets no lever, so the runner shows current law — true, and indistinguishable from a
   * bill that costs nothing unless the page says otherwise. `Priced::cost` returns `None` rather
   * than zero for the same reason one layer down.
   */
  const draft = bundle.drafts.find((d) => d.slug === "hb-643-136-introduced");
  expect(draft, "the introduced bill should be in the feed").toBeDefined();
  expect(draft!.provisions.every((p) => p.lever === "")).toBe(true);

  const html = renderDraft(panel(), draftLevers(draft!, MODEL, BASE_YEAR), draft!.slug);
  expect(html).toContain("draft-unpriceable");
  expect(flat(html)).toContain("Nothing on this page is this bill");
  expect(flat(html)).toContain("There is no cost of zero to report");
});

test("a draft that prices nothing still leaves the levers at current law", () => {
  // The levers are the mechanism by which the page would otherwise lie: an unpriceable draft that
  // moved one would be attributing a scenario to a bill that does not contain it.
  const draft = bundle.drafts.find((d) => d.slug === "hb-643-136-introduced")!;
  const levers = draftLevers(draft, MODEL, BASE_YEAR);
  expect(levers).toEqual(defaultLevers(MODEL, BASE_YEAR));
});
