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
import { DraftProvisionSchema } from "../../src/lib/schema/feed.ts";
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

test("a field the guarantee rule does not read is not a departure", () => {
  /*
   * `matchesDraft` compared `guaranteeArgument` and `isCurrentLaw` did not, and both were asking
   * "have the levers moved?". So `?draft=hb-96-with-refreshed-inputs&arg=0.7` reported:
   *
   *   These levers no longer match the draft. The figures below are yours rather than the bill's.
   *
   * over tiles identical to the bill's — because that draft's guarantee rule is `as-enacted`, which
   * `toPolicy` builds as `{ kind }` with the number dropped. The retained-share control is hidden
   * in that state, so the reader could not have moved it and cannot see what differs.
   *
   * Both now compare the policy rather than the levers, so a field the formula never reads cannot
   * make two scenarios different.
   */
  const draft = bundle.drafts.find((d) => d.slug === "hb-96-with-refreshed-inputs")!;
  const levers = draftLevers(draft, MODEL, BASE_YEAR);
  expect(levers.guarantee).toBe("as-enacted");
  expect(matchesDraft({ ...levers, guaranteeArgument: 0.7 }, draft, MODEL, BASE_YEAR)).toBe(true);
  expect(flat(renderDraft(panel(), { ...levers, guaranteeArgument: 0.7 }, draft.slug))).not.toContain(
    "no longer match",
  );
});

test("a field the guarantee rule does read is still a departure", () => {
  // The other direction, which is what stops the fix above from being a blanket exemption: under
  // `phase-out` the argument is the policy.
  const draft = bundle.drafts.find((d) => d.slug === "fund-the-plan-and-retire-the-guarantee")!;
  const levers = draftLevers(draft, MODEL, BASE_YEAR);
  expect(levers.guarantee).toBe("phase-out");
  expect(matchesDraft({ ...levers, guaranteeArgument: 0.7 }, draft, MODEL, BASE_YEAR)).toBe(false);
});

test("a draft nothing prices does not also report a cost of zero clauses", () => {
  /*
   * `nothingPriced` and `missing` were emitted independently, so a draft with no priced provision
   * got both, in adjacent sentences:
   *
   *   There is no cost of zero to report; there is no cost.
   *   ... the total below is the cost of 0 clauses, not of the bill.
   *
   * The second was written assuming at least one clause priced, and it reported the zero the first
   * had just refused to report. The unpriced list still renders — it is the substance — but the
   * sentence does not.
   */
  const draft = bundle.drafts.find((d) => d.slug === "hb-643-136-introduced")!;
  const html = flat(renderDraft(panel(), draftLevers(draft, MODEL, BASE_YEAR), draft.slug));
  expect(html).toContain("There is no cost of zero to report");
  expect(html).not.toContain("cost of 0 clause");
  // The provisions themselves are still named.
  expect(html).toContain('data-part="draft-unpriced"');
  expect(html).toContain("EdChoice");
});

test("the unpriced sentence agrees with itself about number", () => {
  // "1 of this draft's 1 provisions are not in any figure" — the branch above it handles the
  // singular and this one did not. Checked on a synthesised two-provision draft so the assertion
  // does not depend on which bills the feed happens to carry.
  const base = bundle.drafts.find((d) => d.slug === "fund-the-plan-and-retire-the-guarantee")!;
  const oneUnpriced = {
    ...base,
    provisions: [base.provisions[0]!, { ...base.provisions[2]!, lever: "" as const }],
  };
  const withDraft = { ...panel(), drafts: [oneUnpriced] };
  const html = flat(renderDraft(withDraft, draftLevers(oneUnpriced, MODEL, BASE_YEAR), base.slug));
  expect(html).toContain("1 of this draft's 2 provisions is not in any figure");
  expect(html).toContain("the cost of 1 clause,");
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

test("an unknown draft says the figures are current law and not that bill's", () => {
  /*
   * This used to assert `""`, on the ground that a card headed "opened from a draft" with no draft
   * in it would be worse than no card. That reasoning was right about the *heading* and wrong about
   * the outcome: rendering nothing left `/scenario?draft=hb-XXX` showing a plain current-law page
   * with `&draft=hb-XXX` still in the address bar, so a reader who followed a link to a bill met
   * figures they had every reason to read as the bill's.
   *
   * Which is the argument `renderDraft` already makes about the departure banner — "a banner that
   * vanished would leave the reader with a number they still believe is the bill's, which is worse
   * than no banner at all". A slug is exactly the thing that goes stale in a shared link.
   *
   * So the card is rendered and its heading names the real state instead.
   */
  const html = flat(renderDraft(panel(), defaultLevers(MODEL, BASE_YEAR), "no-such-bill"));
  expect(html).toContain("That bill is not in this feed");
  expect(html).toContain("no-such-bill");
  expect(html).toContain("The figures below are current law");
  expect(html).toContain('data-part="draft-unknown"');
  // Not the card that describes a bill this feed does carry.
  expect(html).not.toContain("Opened from a draft");
});

test("an unknown slug is escaped rather than interpolated", () => {
  // The slug is the one part of this card that comes from the URL bar.
  const html = renderDraft(panel(), defaultLevers(MODEL, BASE_YEAR), '<img src=x onerror=alert(1)>');
  expect(html).not.toContain("<img");
  expect(html).toContain("&lt;img");
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

test("a lever-bearing provision whose value does not parse is rejected at the schema", () => {
  /*
   * `Number("1.04x")` is `NaN`, and a `NaN` reaching `baseCostScale` turns every figure on the
   * scenario page into `$NaN` across 609 districts — under a banner saying these are the bill's
   * numbers. `draftLevers` guards it too, but the schema is the boundary between a Rust guarantee
   * and a JSON file, and a value that cannot be a lever position should not get past it.
   */
  const ok = {
    ordinal: 1,
    title: "Base cost reference year",
    authority: "R.C. 3317.011",
    parameter: "",
    lever: "base-cost" as const,
    proposed: "1.0395",
    note: "sized against the FY2024 restatement",
  };
  expect(DraftProvisionSchema.safeParse(ok).success).toBe(true);
  expect(DraftProvisionSchema.safeParse({ ...ok, proposed: "1.04x" }).success).toBe(false);
  expect(DraftProvisionSchema.safeParse({ ...ok, proposed: "" }).success).toBe(false);

  // An unpriced provision says whatever it needs to say — it sets no lever.
  expect(
    DraftProvisionSchema.safeParse({ ...ok, lever: "", proposed: "each weight times 1.08" }).success,
  ).toBe(true);

  // A guarantee provision names a rule rather than a number.
  const guarantee = { ...ok, lever: "guarantee" as const, proposed: "phase-out:0.5" };
  expect(DraftProvisionSchema.safeParse(guarantee).success).toBe(true);
  expect(DraftProvisionSchema.safeParse({ ...guarantee, proposed: "abolished" }).success).toBe(false);
});

test("a provision the schema would reject cannot reach the levers either", () => {
  // The second line, because the schema is one process away. `draftLevers` leaves the default
  // standing rather than writing `NaN` into it.
  const draft = {
    slug: "synthetic",
    provisions: [
      { ordinal: 1, title: "t", authority: "a", parameter: "", lever: "base-cost" as const, proposed: "1.04x", note: "n" },
      { ordinal: 2, title: "t", authority: "a", parameter: "", lever: "min-share" as const, proposed: "", note: "n" },
    ],
  };
  const levers = draftLevers(draft, MODEL, BASE_YEAR);
  expect(Number.isFinite(levers.baseCostScale)).toBe(true);
  expect(levers.baseCostScale).toBe(1);
  expect(levers.minimumStateShare).toBe(MODEL);
});
