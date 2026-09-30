/**
 * A bill in the runner, and a bill nothing here can price.
 *
 * Both halves of the same honesty: a draft whose parameters this model holds gets run and shown,
 * and a draft whose mechanism it does not gets said so rather than approximated. The second block
 * exists because the tempting failure is to price everything.
 *
 * Everything here goes through `/scenario?draft=…`, which re-enters the runner and *undoes* an edit
 * made before the panel landed rather than ignoring it. That is #219, and the note on `live` below
 * is the whole of it — read it before adding a test to this file.
 */

import { expect, test, type Page } from "@playwright/test";

import { NORTHERN } from "./helpers.ts";

test.describe("a draft opened in the runner", () => {
  /**
   * The runner is live: its panel has arrived and its levers are wired.
   *
   * Nothing may touch a control before this resolves, and on a draft URL that is not a matter of
   * patience. `src/scripts/scenario-load.ts` boots the runner once `fetch(data/panel.json)` lands,
   * and until then `#scenario-out` is empty and no `input` listener exists — so a control is an `<input>` and not
   * a lever. `boot` then calls `put()` for every lever the draft sets and renders with
   * `fromControls: false`, on purpose, because the sliders are quantized and the bill's values are
   * not. An edit made before boot is therefore not ignored. It is **undone**.
   *
   * That is #219. The test below waited on `[data-part="draft-departed"]` having count zero, which
   * the empty pre-boot page satisfies exactly as well as the loaded one does, and then moved a
   * lever into a page with nothing listening. It failed twice in full-suite runs, where the fetch
   * competes with three other workers, and never in isolation or on CI.
   */
  async function runnerIsLive(page: Page): Promise<void> {
    await expect(page.locator('.card[data-part="draft"]')).toContainText("Opened from a draft");
  }

  test("the unpriced provisions are on screen with the total, not below it", async ({ page }) => {
    /*
     * The rule this whole class rests on, at the layer where it is easiest to lose. `crates/project`
     * cannot print a bill's cost without the clauses it failed to price; the feed carries them; and
     * this is the check that the page actually shows them beside the number rather than somewhere a
     * reader scrolls past.
     */
    await page.goto("/scenario?draft=fund-the-plan-and-retire-the-guarantee");
    const card = page.locator('.card[data-part="draft"]');
    await expect(card).toContainText("Opened from a draft");
    await expect(card).toContainText("not in any figure on this page");
    await expect(page.locator('[data-part="draft-unpriced"] li')).toHaveCount(2);

    // First, not merely present: the placement the rule requires is a limit on what is about to be
    // read rather than a footnote on what was read.
    await expect(page.locator("#scenario-out > :first-child")).toHaveAttribute(
      "data-part",
      "draft",
    );
  });

  test("the figure is the draft's even where a slider cannot express it", async ({ page }) => {
    /*
     * `#lv-base` steps by 0.01 and the refresh provision is 1.0395, so the control can only get
     * to 1.04, which is a different scenario from the bill's. The controls are set as near as
     * they go and the first render is computed from the draft, so the number under the banner is
     * the bill's rather than the slider's.
     */
    await page.goto("/scenario?draft=fund-the-plan-and-retire-the-guarantee");
    await expect(page.locator("#lv-guarantee")).toHaveValue("phase-out");
    await expect(page.locator("#lv-arg")).toHaveValue("0.5");
    await expect(page.locator("#lv-base")).toHaveValue("1.04");

    // The combined figure, and not the one the rounded slider would give.
    await expect(page.locator("#scenario-out")).toContainText("\u2212$325.6M");
    await expect(page.locator("#scenario-out")).toContainText("297 up, 312 down");
    await expect(page.locator('[data-part="draft-departed"]')).toHaveCount(0);
  });

  test("moving a lever says the figures are no longer the bill's", async ({ page }) => {
    // The likely failure: a reader nudges a slider and the page goes on attributing the number to
    // the draft. Removing the banner instead would be worse — a figure they still believe is the
    // bill's, with nothing to correct them.
    await page.goto("/scenario?draft=fund-the-plan-and-retire-the-guarantee");
    await runnerIsLive(page);
    await expect(page.locator('[data-part="draft-departed"]')).toHaveCount(0);

    await page.locator("#lv-base").fill("1.08");
    await page.locator("#lv-base").dispatchEvent("input");

    await expect(page.locator('[data-part="draft-departed"]')).toContainText(
      "no longer match the draft",
    );
    // And the missing clauses stay put: a departed scenario is short the same three provisions.
    await expect(page.locator('[data-part="draft-unpriced"] li')).toHaveCount(2);
  });

  test("a lever moved before the runner is live is undone, not obeyed", async ({ page }) => {
    /*
     * The behaviour the test above depends on, pinned rather than assumed, and the reason #219
     * could not be closed by adding a `retry`.
     *
     * The panel is held back so the window is a certainty instead of a race — 800ms against a
     * fetch that normally takes single-digit milliseconds. The slider is moved into that window,
     * where nothing is listening, and the assertions are that `boot` puts it back and renders the
     * *bill's* figure under the banner that says it is the bill's.
     *
     * `−$325.6M` is what makes this worth a test rather than a comment. The control cannot express
     * the refresh provision's 1.0395 — it steps by 0.01 — so a runner that read its controls for
     * the first render would publish −$139.9M under "Opened from a draft". That is the one thing
     * `fromControls: false` exists to prevent, and it is invisible in every other test here
     * because every other test lets the page settle first.
     */
    await page.route("**/data/panel.json", async (route) => {
      await new Promise((resolve) => setTimeout(resolve, 800));
      await route.continue();
    });
    await page.goto("/scenario?draft=fund-the-plan-and-retire-the-guarantee");

    // Into the window. Without the route above this line is a coin flip, which is what #219 was.
    await page.locator("#lv-base").fill("1.08");
    await page.locator("#lv-base").dispatchEvent("input");

    await runnerIsLive(page);
    await expect(page.locator("#lv-base"), "the draft put its own value back").toHaveValue("1.04");
    await expect(
      page.locator('[data-part="draft-departed"]'),
      "nothing departed, because nothing the reader did survived",
    ).toHaveCount(0);
    await expect(
      page.locator("#scenario-out"),
      "and the figure is the bill's, not the one the quantized slider can reach",
    ).toContainText("\u2212$325.6M");
  });

  test("a draft that prices completely says so rather than staying silent", async ({ page }) => {
    await page.goto("/scenario?draft=hb-96-with-refreshed-inputs");
    await expect(page.locator('[data-part="draft-complete"]')).toContainText(
      "a property of a one-clause draft",
    );
    await expect(page.locator('[data-part="draft-unpriced"]')).toHaveCount(0);
  });

  test("the draft's node page states the priced ratio beside the link, not just the link", async ({
    page,
  }) => {
    // An invitation to see a bill's cost carries, at the point of the invitation, the fact that the
    // cost is of some of its clauses. Third place the same refusal is made, and the first a reader
    // meets.
    await page.goto("/wiki/draft-legislation/fund-the-plan-and-retire-the-guarantee");
    const card = page.locator('.card[data-part="runner"]');
    await expect(card).toContainText("4 of this draft's 6 provisions");
    await expect(card).toContainText("not of the bill");
    await expect(card.locator("a.pill")).toHaveAttribute(
      "href",
      "/scenario?draft=fund-the-plan-and-retire-the-guarantee",
    );
  });

  test("the district route carries the draft card too, and it is first", async ({ page }) => {
    /*
     * The failure a review caught: `?draft=` was read on both routes but the card was written only
     * in the statewide branch, so a district page applied a bill's levers and reported a dollar
     * and per-pupil figure with nothing naming the three provisions it could not price. A district
     * page is the one a school board sends, which makes it the worst place for that and not the
     * mildest.
     */
    await page.goto(
      `/scenario?d=${NORTHERN}&draft=fund-the-plan-and-retire-the-guarantee`,
    );
    await expect(page.locator("#scenario-out > :first-child")).toHaveAttribute(
      "data-part",
      "draft",
    );
    await expect(page.locator('[data-part="draft-unpriced"] li')).toHaveCount(2);
    // And the district figures are still there below it.
    await expect(page.locator("#scenario-out")).toContainText("What moved for this district");
  });

  test("an unknown draft leaves the runner working rather than showing an empty card", async ({
    page,
  }) => {
    await page.goto("/scenario?draft=no-such-bill");
    await expect(page.locator('.card[data-part="draft"]')).toHaveCount(0);
    await expect(page.locator("#scenario-out")).toContainText("Current law");
  });
});

test.describe("a pending bill nothing here can price", () => {
  test("the page says it is not the bill rather than showing current law", async ({ page }) => {
    /*
     * H.B. 643 of the 136th caps EdChoice expansion eligibility, and every provision it has falls
     * in the scholarship channel this repository does not model. It sets no lever, so the runner
     * shows current law — true, and indistinguishable from a bill that costs nothing unless the
     * page says which it is.
     */
    await page.goto("/scenario?draft=hb-643-136-introduced");
    const card = page.locator('.card[data-part="draft"]');
    await expect(card).toContainText("Nothing on this page is this bill");
    await expect(card).toContainText("There is no cost of zero to report");
    await expect(page.locator('[data-part="draft-unpriced"] li')).toHaveCount(1);
  });

  test("its node page offers no invitation to a figure that does not exist", async ({ page }) => {
    await page.goto("/wiki/draft-legislation/hb-643-136-introduced");
    const card = page.locator('.card[data-part="runner"]');
    await expect(card).toContainText("0 of this draft's 1 provisions");
    await expect(card).toContainText("not of the bill");
  });
});
