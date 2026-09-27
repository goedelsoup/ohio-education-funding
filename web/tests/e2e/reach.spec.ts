/**
 * Every figure a reader meets can reach the year it is measured in.
 *
 * The browser half of the reach rule. `figures.spec.ts` sweeps the built pages for figures with no
 * year within reach; this holds the cases that only exist once the page has run — a chip rendered
 * from a lever, a column head redrawn by a sort, a tooltip's own dating.
 *
 * Chart selectors here say `svg.plot:visible`; the reason is in `helpers.ts`.
 */

import { expect, test, type Page } from "@playwright/test";

/*
 * The runner's two views, and the one thing that has to be true of a split.
 *
 * `/scenario` and `/reach` run the same levers over the same panel and ask different questions of
 * the answer — how much, and who. That only pays for itself if each page asks one of them and the
 * reader can cross between them without losing their settings, which is what these assert.
 */
test.describe("reach", () => {
  /**
   * Wait for the runner to have booted before touching a control.
   *
   * The scenario routes fetch the panel, verify it against the Rust checkpoints, and only then
   * attach their listeners — so a click or a drag that lands before that is an input event with
   * nothing listening, and the page afterwards looks exactly as if the reader had never touched
   * it. On the preset row it is worse than a no-op: the pressed state is server-rendered on
   * "Current law", so a swallowed click leaves exactly one button pressed and it is the wrong one,
   * which passes a count assertion and fails a text one.
   *
   * The cloud is the signal because it is written by the same `render()` the controls call.
   */
  const booted = (page: Page) =>
    expect(page.locator('#scenario-out [data-part="positions"], #scenario-out .card')).not.toHaveCount(0);

  test("the cloud is drawn before any lever moves, and the scenario's is gone", async ({ page }) => {
    /*
     * The case for the page being a page. Every other thing the runner draws is blank at rest —
     * under current law nothing has moved, so there is no distribution to bin and no ranking to
     * make — and this one is not, because the guarantee wall is a property of the formula rather
     * than of a lever position. A reader who lands here from the bar sees the shape of the state.
     */
    await page.goto("/reach");
    await booted(page);
    await expect(page.locator('#scenario-out [data-part="positions"]')).toBeVisible();
    // `:visible` because every chart is drawn twice, at both container widths, and the stylesheet
    // picks one. See `renderToString`.
    await expect(page.locator('[data-chart="positions"] svg.plot:visible')).toBeVisible();

    // And it is not on both pages. A split that leaves the card behind is two copies to keep true.
    await page.goto("/scenario");
    await booted(page);
    await page.locator("#lv-base").fill("1.05");
    await page.locator("#lv-base").dispatchEvent("input");
    await expect(page.locator('[data-part="distribution"]')).toBeVisible();
    await expect(page.locator('[data-part="positions"]')).toHaveCount(0);
  });

  test("crossing between the two views keeps the levers", async ({ page }) => {
    /*
     * The failure this closes is silent: a link to the other question that drops the settings
     * offers the reader that question about a scenario they are no longer looking at, and the page
     * it lands on looks perfectly correct — it is just answering about current law.
     */
    await page.goto("/scenario");
    await booted(page);
    await page.locator("#lv-base").fill("1.12");
    await page.locator("#lv-base").dispatchEvent("input");
    await page.locator('[data-part="distribution"] a[data-carry-levers]').click();

    await expect(page).toHaveURL(/\/reach\?.*base=1\.12/);
    await expect(page.locator("#lv-base")).toHaveValue("1.12");
    await expect(page.locator('#scenario-out [data-part="positions"]')).toBeVisible();

    // And back the other way, from the lever position the first leg arrived at.
    await page.locator('[data-part="positions"] a[data-carry-levers]').click();
    await expect(page).toHaveURL(/\/scenario\?.*base=1\.12/);
    await expect(page.locator("#lv-base")).toHaveValue("1.12");
    await expect(page.locator('[data-part="outcome"]')).toBeVisible();
  });

  test("the axes hold still while the levers move", async ({ page }) => {
    /*
     * The defect this closes, which is the one a movement chart cannot have.
     *
     * The axes were fitted to the run being drawn, so dragging base cost held the cloud still and
     * moved the *frame* underneath it — and a reader cannot tell from that whether the districts
     * moved or the ruler did. It also made the trails meaningless: a segment between two positions
     * only says something if the two are measured on one scale.
     *
     * Read off the drawn labels rather than off the domain, because the label is what the reader
     * actually compares between ticks.
     */
    const ends = async () => {
      const html = await page.locator('[data-chart="positions"]').innerHTML();
      return [...html.matchAll(/<text[^>]*>([^<]*)<\/text>/g)].map((m) => m[1]).join("|");
    };

    await page.goto("/reach");
    await booted(page);
    const frame = await ends();
    expect(frame).toContain("$0");

    for (const value of ["0.8", "1.3"]) {
      await page.locator("#lv-base").fill(value);
      await page.locator("#lv-base").dispatchEvent("input");
      expect(await ends(), `base cost ${value} moved the frame`).toBe(frame);
    }
    // And the two levers that reshape the picture hardest: the floor removed, and the floor under
    // the state share raised to its ceiling.
    await page.locator("#lv-guarantee").selectOption("removed");
    await page.locator("#lv-guarantee").dispatchEvent("input");
    expect(await ends(), "removing the guarantee moved the frame").toBe(frame);
    await page.locator("#lv-min").fill("0.3");
    await page.locator("#lv-min").dispatchEvent("input");
    expect(await ends(), "the minimum state share moved the frame").toBe(frame);
  });

  test("the districts the frame cannot hold are counted, not fitted to", async ({ page }) => {
    /*
     * Kelleys Island Local has four pupils. Its aid per pupil is $26,700 against a 99th percentile
     * of $15,305 and its base cost per pupil is $371,449 against $12,221 — so an axis fitted to it
     * makes the other six hundred districts a smudge against the left edge. The frame is a
     * quantile and the marks are clipped to it, which is only honest if the page says how many it
     * clipped.
     */
    await page.goto("/reach");
    await booted(page);
    const notes = page.locator('[data-part="positions"] .note');
    await expect(notes.filter({ hasText: "sit outside the frame" })).toHaveCount(1);
    await expect(notes.filter({ hasText: "Kelleys Island" })).toHaveCount(1);

    // A share cannot leave 0–1, so that frame holds everybody and the note is absent rather than
    // reading "0 districts" — an absence stated is worse than an absence.
    await page.locator("#rv-x").selectOption("poverty");
    await page.locator("#rv-y").selectOption("stateShare");
    await expect(notes.filter({ hasText: "sit outside the frame" })).toHaveCount(0);
  });

  test("choosing an axis changes the drawing and takes the wall with it", async ({ page }) => {
    /*
     * The identity line asserts `y ≥ x` as a law, and that is true of realized against formula aid
     * and of no other pair the menu can make. Left drawn on, say, valuation against poverty it
     * would be a reference line through a cloud that means nothing — which is the one thing
     * `scatterSpec`'s own note on `identity` says this form must never do. So it is gated on the
     * pair rather than on a flag, and this is the check that the gate is on the pair.
     */
    await page.goto("/reach");
    await booted(page);
    await expect(page.locator(".scatter-identity")).not.toHaveCount(0);

    await page.locator("#rv-x").selectOption("valuation");
    await page.locator("#rv-y").selectOption("poverty");
    await expect(page.locator(".scatter-identity")).toHaveCount(0);
    await expect(page.locator('[data-chart="positions"] svg.plot:visible circle')).not.toHaveCount(0);
    // And the pair travels, so a cloud is worth sending somebody in the axes it was read in.
    await expect(page).toHaveURL(/vx=valuation/);
    await expect(page).toHaveURL(/vy=poverty/);

    await page.locator("#rv-x").selectOption("formula");
    await page.locator("#rv-y").selectOption("realized");
    await expect(page.locator(".scatter-identity")).not.toHaveCount(0);
  });

  test("a preset is a position, so exactly one of them is ever pressed", async ({ page }) => {
    /*
     * They were patches first — each naming only the fields it changed, so two could compose — and
     * the row then reported three buttons pressed at once under a heading reading "Start from".
     * Three simultaneous selections is a lie about what the reader is looking at.
     *
     * "Fund the plan" is also the one whose value the sliders cannot hold exactly: the feed prices
     * the refresh at 1.0395 and the control steps by 0.01, so a preset compared by equality would
     * read unpressed the instant it was pressed.
     */
    await page.goto("/reach");
    await booted(page);
    const pressed = page.locator('[data-preset][aria-pressed="true"]');
    await expect(pressed).toHaveCount(1);
    await expect(pressed).toContainText("Current law");

    await page.locator('.preset:has-text("Fund the plan")').click();
    await expect(pressed).toHaveCount(1);
    await expect(pressed).toContainText("Fund the plan");
    expect(Number(await page.locator("#lv-base").inputValue())).toBeGreaterThan(1);

    await page.locator('.preset:has-text("Retire half the floor")').click();
    await expect(pressed).toHaveCount(1);
    await expect(pressed).toContainText("Retire half the floor");
    // A position, not a patch: the base cost the previous preset set is back at current law.
    await expect(page.locator("#lv-base")).toHaveValue("1");
    await expect(page.locator("#lv-guarantee")).toHaveValue("phase-out");

    // And a slider nudged off a preset stops reading as one, because the state is derived.
    await page.locator("#lv-min").fill("0.2");
    await page.locator("#lv-min").dispatchEvent("input");
    await expect(pressed).toHaveCount(0);
  });

  test("turning the trails off leaves the cloud and removes the segments", async ({ page }) => {
    await page.goto("/reach");
    await booted(page);
    await page.locator("#lv-base").fill("1.12");
    await page.locator("#lv-base").dispatchEvent("input");
    await expect(page.locator(".scatter-trail path")).not.toHaveCount(0);

    await page.locator("#rv-trails").uncheck();
    await expect(page.locator(".scatter-trail path")).toHaveCount(0);
    await expect(page.locator('[data-chart="positions"] svg.plot:visible circle')).not.toHaveCount(0);
    await expect(page).toHaveURL(/t=0/);
  });

  test("a district that the guarantee holds draws a trail that runs flat", async ({ page }) => {
    /*
     * The finding the page exists for, asserted on the drawing rather than on the model.
     *
     * `scatterSpec` emits a displacement segment only for the points that moved, so under a base
     * cost rise there is one per district whose formula amount changed — which is every district,
     * held or not. What separates the two is whether the segment is level: a held district's
     * payment does not move, so `y1 === y2`. If that ever stopped being true the picture would
     * still look plausible and would be claiming the guarantee reaches people it does not.
     */
    await page.goto("/reach");
    await booted(page);
    await page.locator("#lv-base").fill("1.12");
    await page.locator("#lv-base").dispatchEvent("input");
    // Plot's `className` lands on the mark's `<g>`; the segments themselves are `<path>`, one per
    // district that moved, each a two-point `M…L…`.
    await expect(page.locator(".scatter-trail path")).not.toHaveCount(0);

    const flat = await page.locator(".scatter-trail path").evaluateAll((nodes) =>
      nodes.filter((n) => {
        const pairs = [...(n.getAttribute("d") ?? "").matchAll(/(-?[\d.]+),(-?[\d.]+)/g)];
        if (pairs.length < 2) return false;
        const first = pairs[0]!;
        const last = pairs[pairs.length - 1]!;
        return Math.abs(Number(first[2]) - Number(last[2])) < 0.5;
      }).length,
    );
    expect(flat, "a base-cost rise has to leave some held district exactly where it was")
      .toBeGreaterThan(0);
  });

  /*
   * The scope, which is the reader's own question: not *which districts does this reach* but *does
   * it reach mine*.
   *
   * A selection lights rather than subsets, and the reason is measured rather than preferred.
   * `scatterSpec` refuses fewer than twelve points — a scatter of three districts reads as a finding
   * about a population nobody measured — and 79 of Ohio's 88 counties hold fewer than twelve
   * districts, so a filter that drew only the selection would draw a blank card for almost every
   * county in the state. `reach.spec.ts` holds that measurement; these hold what a reader gets.
   */
  test("a county selection lights its districts and leaves the state drawn behind them", async ({
    page,
  }) => {
    await page.goto("/reach");
    await booted(page);
    const drawn = page.locator('[data-chart="positions"] .chart-at[data-at="wide"] .scatter-dot circle');
    const before = await drawn.count();
    expect(before, "the whole state is in the cloud at rest").toBeGreaterThan(600);

    await page.locator("#rv-counties > summary").click();
    await page.locator('#reach-scope input[name="co"][value="athens"]').check();

    // Every district is still plotted. This is the assertion a subsetting filter fails.
    expect(await drawn.count(), "a selection must not remove districts from the cloud").toBe(before);
    /*
     * Two radii, because opacity alone could not carry this: `--neutral-mark` is close to its
     * surface by design, so muting it separates the two populations by 1.20:1 — not a difference a
     * reader finds in six hundred overlapping dots. See `DOT` in `plot/spec.ts`.
     */
    const lit = page.locator('[data-chart="positions"] .chart-at[data-at="wide"] .scatter-dot circle[r="2.4"]');
    const context = page.locator('[data-chart="positions"] .chart-at[data-at="wide"] .scatter-dot circle[r="1.6"]');
    expect(await lit.count(), "the county's districts are the subject").toBeGreaterThan(0);
    expect(await lit.count()).toBeLessThan(12);
    expect(await context.count(), "and the rest are context").toBeGreaterThan(500);

    // The legend says which, and it says it on the channel the dots actually use.
    await expect(page.locator('[data-part="positions"] .legend')).toContainText(
      "Outside Athens County",
    );
    await expect(page.locator('[data-part="positions"] .legend .sw.muted')).toHaveCount(1);
  });

  test("every count a reader reads as an answer is restated against the selection", async ({
    page,
  }) => {
    /*
     * The prose cost of a spotlight, and the thing it would be easy to skip. "253 of 609 are paid
     * the same" and "4 of 6 in Athens County are" are different claims, and a page that narrowed the
     * picture without narrowing the sentence would be answering about a population the reader is not
     * looking at.
     *
     * The clipped count stays global on purpose: it is a fact about the drawing, and the whole state
     * is still drawn.
     */
    await page.goto("/reach");
    await booted(page);
    const notes = page.locator('[data-part="positions"] .note');
    await expect(notes.first()).toContainText("districts are");

    await page.locator("#rv-counties > summary").click();
    await page.locator('#reach-scope input[name="co"][value="athens"]').check();

    await expect(notes.first()).toContainText("districts in Athens County are");
    await expect(
      notes.filter({ hasText: "muted, and they are not in any count above" }),
      "the districts not in scope are accounted for rather than quietly dropped",
    ).toHaveCount(1);
    await expect(notes.filter({ hasText: "sit outside the frame" })).toContainText("of 609");
  });

  test("a selection of one district still gets a cloud, and the state's own frame", async ({
    page,
  }) => {
    /*
     * The case that decided the design twice over. One district is not twelve points, so a filter
     * would have rendered nothing here; and the frame is the state's, so the one lit dot sits where
     * the state puts it rather than alone on an axis fitted to itself.
     */
    await page.goto("/reach");
    await booted(page);
    const ends = async () => {
      const html = await page.locator('[data-chart="positions"]').innerHTML();
      return [...html.matchAll(/<text[^>]*>([^<]*)<\/text>/g)].map((m) => m[1]).join("|");
    };
    const frame = await ends();

    await page.locator("#rv-add").selectOption("043786");
    await expect(page.locator('[data-chart="positions"] svg.plot:visible circle')).not.toHaveCount(0);
    expect(await ends(), "a selection moved the frame").toBe(frame);
    await expect(
      page.locator('[data-chart="positions"] .chart-at[data-at="wide"] .scatter-dot circle[r="2.4"]'),
    ).toHaveCount(1);
  });

  test("a selection travels in the query string and comes back as chips", async ({ page }) => {
    await page.goto("/reach");
    await booted(page);
    await page.locator("#rv-counties > summary").click();
    await page.locator('#reach-scope input[name="co"][value="athens"]').check();
    await page.locator("#rv-add").selectOption("043786");

    await expect(page).toHaveURL(/co=athens/);
    await expect(page).toHaveURL(/d=043786/);
    /* The picker returns to its placeholder. Left showing the last choice it would read as *the*
       selection while the chips said otherwise. */
    await expect(page.locator("#rv-add")).toHaveValue("");

    await page.reload();
    await booted(page);
    await expect(page.locator('#reach-scope input[name="co"][value="athens"]')).toBeChecked();
    await expect(page.locator("#rv-chips button[data-county='athens']")).toHaveCount(1);
    await expect(page.locator("#rv-chips button[data-district='043786']")).toContainText(
      "Cleveland",
    );

    // A chip is a thing you can take off, and taking the last one off is the whole state again.
    await page.locator("#rv-chips button[data-district='043786']").click();
    await expect(page).not.toHaveURL(/d=043786/);
    await page.locator("#rv-clear").click();
    await expect(page.locator("#rv-chips")).toBeHidden();
    await expect(page).not.toHaveURL(/co=athens/);
    await expect(page.locator('#reach-scope input[name="co"][value="athens"]')).not.toBeChecked();
  });

  test("the selection survives a lever move and a preset", async ({ page }) => {
    /*
     * A scope is not lever state — it says who the picture is about, not what the formula computes —
     * so a preset, which is a whole lever position, must not take it with it. The two are read from
     * different controls and this is what says they stay that way.
     */
    await page.goto("/reach");
    await booted(page);
    await page.locator("#rv-counties > summary").click();
    await page.locator('#reach-scope input[name="co"][value="athens"]').check();

    await page.locator("#lv-base").fill("1.12");
    await page.locator("#lv-base").dispatchEvent("input");
    await expect(page.locator("#rv-chips button[data-county='athens']")).toHaveCount(1);

    await page.locator('.preset:has-text("Retire half the floor")').click();
    await expect(page.locator("#lv-guarantee")).toHaveValue("phase-out");
    await expect(
      page.locator("#rv-chips button[data-county='athens']"),
      "a preset is a lever position and says nothing about who is in scope",
    ).toHaveCount(1);
    await expect(page).toHaveURL(/co=athens/);
  });

  test("the mode control appears with a selection and not before it", async ({ page }) => {
    /*
     * A control that changes nothing is worse than no control, because it reads as one whose effect
     * the reader has failed to notice. The same rule the typology picker is hidden by, and the
     * reason the mode lives in the scope form rather than beside the axis menus: it modifies the
     * selection above it and means nothing without one.
     */
    await page.goto("/reach");
    await booted(page);
    await expect(page.locator("#rv-mode-pick")).toBeHidden();

    await page.locator("#rv-counties > summary").click();
    await page.locator('#reach-scope input[name="co"][value="cuyahoga"]').check();
    await expect(page.locator("#rv-mode-pick")).toBeVisible();
    await expect(
      page.locator('#reach-scope input[name="mode"][value="spotlight"]'),
      "the spotlight is forced, because it is the only mode that can draw any selection",
    ).toBeChecked();

    /* "Every district" says it lights every district, so it puts the mode back too — otherwise the
       next county a reader picked would be silently subsetted, and for 79 of 88 that is a refusal. */
    await page.locator('#reach-scope input[name="mode"][value="subset"]').check();
    await page.locator("#rv-clear").click();
    await expect(page.locator("#rv-mode-pick")).toBeHidden();
    await expect(page.locator('#reach-scope input[name="mode"][value="spotlight"]')).toBeChecked();
  });

  test("a subset drops the rest of the state and keeps the state's frame", async ({ page }) => {
    /*
     * The second picture, and the one the spotlight cannot draw: 31 districts among 609 are 31 dots
     * in a crowd however they are shaded, so *how are these arranged among themselves* needs the
     * rest gone. What must not go with them is the ruler — a frame fitted to one county would make
     * two counties incomparable, and silently, because a refitted axis looks like an axis.
     */
    await page.goto("/reach");
    await booted(page);
    const dots = page.locator('[data-chart="positions"] .chart-at[data-at="wide"] .scatter-dot circle');
    const ends = async () => {
      const html = await page.locator('[data-chart="positions"]').innerHTML();
      return [...html.matchAll(/<text[^>]*>([^<]*)<\/text>/g)].map((m) => m[1]).join("|");
    };
    const frame = await ends();
    expect(await dots.count()).toBeGreaterThan(600);

    await page.locator("#rv-counties > summary").click();
    await page.locator('#reach-scope input[name="co"][value="cuyahoga"]').check();
    await page.locator('#reach-scope input[name="mode"][value="subset"]').check();

    expect(await dots.count(), "only the county is drawn").toBe(31);
    expect(await ends(), "and on the frame the whole state is measured on").toBe(frame);
    // Nothing is out of scope on the chart, so the legend must not name a population that is gone.
    await expect(page.locator('[data-part="positions"] .legend .sw.muted')).toHaveCount(0);
    await expect(page.locator('[data-part="positions"] .legend')).not.toContainText("for context");

    /*
     * The identity line is a law and not a fit — a district is paid the larger of its formula
     * amount and its guarantee — so it holds for 31 districts exactly as for 609 and is gated on
     * the pair of axes rather than on the population.
     */
    await expect(page.locator(".scatter-identity")).not.toHaveCount(0);

    // Every count restated against the selection, including the two the spotlight leaves global.
    const card = page.locator('[data-part="positions"]');
    const notes = page.locator('[data-part="positions"] .note');
    await expect(notes.first()).toContainText("districts in Cuyahoga County are");
    await expect(notes.filter({ hasText: "Only Cuyahoga County is drawn" })).toHaveCount(1);
    await expect(notes.filter({ hasText: "muted, and they are not in any count above" })).toHaveCount(0);
    await expect(card, "no count on this card is against a population it does not draw")
      .not.toContainText("of 609");

    // And it is a link worth sending, so it travels.
    await expect(page).toHaveURL(/only=1/);
    await page.reload();
    await booted(page);
    await expect(page.locator('#reach-scope input[name="mode"][value="subset"]')).toBeChecked();
    expect(await dots.count()).toBe(31);
  });

  test("a subset too small to be a cloud refuses, and offers the spotlight", async ({ page }) => {
    /*
     * The case the whole design turns on. Athens County holds five districts; `scatterSpec` returns
     * null under twelve points and a null spec renders to the empty string, so the alternative to
     * refusing is a heading with a gap under it. The refusal has to say why in that rule's own
     * terms and has to have somewhere to send the reader — which is what the spotlight is for.
     */
    await page.goto("/reach?co=athens&only=1");
    await booted(page);
    const card = page.locator('[data-part="positions"]');
    await expect(card).toContainText("Athens County is too small to draw on its own");
    await expect(card, "the floor, said as the reason it is a floor").toContainText(
      "a population nobody measured",
    );
    await expect(card, "and how rare that is, counted rather than typed").toContainText(
      "9 of Ohio's 88 counties",
    );
    await expect(
      page.locator('[data-chart="positions"]'),
      "a refusal, not a blank card with a legend over it",
    ).toHaveCount(0);

    /* The way out is the control, not an instruction to go and find one. */
    await page.locator('[data-part="positions"] button[data-scope-mode="spotlight"]').click();
    await expect(page.locator('#reach-scope input[name="mode"][value="spotlight"]')).toBeChecked();
    await expect(page).not.toHaveURL(/only=1/);
    await expect(
      page.locator('[data-chart="positions"] .chart-at[data-at="wide"] .scatter-dot circle[r="2.4"]'),
      "and the selection the subset refused is lit in the mode that can draw it",
    ).toHaveCount(5);
  });

  test("the spotlight offers the subset only where the selection can support one", async ({
    page,
  }) => {
    /*
     * The mode radio can be set at any selection; this sentence is the answer to *can I?*, which
     * the radio cannot give. So it appears under a county that holds enough districts and not under
     * one that does not — a reader must not be invited into a refusal.
     */
    await page.goto("/reach?co=athens");
    await booted(page);
    const offer = page.locator('[data-part="positions"] button[data-scope-mode="subset"]');
    await expect(offer).toHaveCount(0);

    await page.goto("/reach?co=cuyahoga");
    await booted(page);
    await expect(offer).toHaveCount(1);
    await offer.click();
    await expect(page.locator('#reach-scope input[name="mode"][value="subset"]')).toBeChecked();
    await expect(
      page.locator('[data-chart="positions"] .chart-at[data-at="wide"] .scatter-dot circle'),
    ).toHaveCount(31);
  });

});
