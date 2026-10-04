/**
 * The preview cards for the pages that are not one of something.
 *
 * A district card is generated from a district. There are 609 of them and no table names any of
 * them. `/method`, `/outcomes`, `/history` and their neighbours are the opposite case: fourteen
 * pages, each about a different thing, each needing a sentence written for it once.
 *
 * The figures are still read from the feed rather than typed in. A card saying "609 districts" that
 * a regenerated panel makes wrong is worse than one saying nothing, because it is shared onward
 * without the page beside it to correct it — the same reasoning that put `TaxStatewide` in
 * `feed.ts` instead of in prose.
 *
 * The keys are what `routes.og.page()` takes, and they are route paths minus the slash. `/` is
 * the exception and takes no key at all: it is the site's own card, `og.default()`, because the
 * homepage previews as the site rather than as one of these. `"statewide"` was that exception's
 * workaround for as long as the statewide panel *was* the root — the key outlived the arrangement
 * by about ten minutes and now names the route it always claimed to.
 */

import { census } from "../bounds.ts";
import { loadCorpus } from "../corpus.ts";
import { loadFeed } from "../feed.ts";
import { acts, regimes, regimesCrossed } from "../legislation.ts";
import { count, millions, pct } from "../format.ts";
import { clamp, type Card } from "./card.ts";
import { firstOf, lastOf } from "../ends.ts";
import { statewideLevels } from "../whatChanged.ts";
import { published } from "../explained/registry.ts";

/** The eyebrow every card wears unless it has a better one. */
export const SITE = "Ohio school funding";

/**
 * Every card that is written rather than derived, keyed by slug.
 *
 * Built on call rather than at module scope: `loadFeed` throws when the formula check fails, and a
 * module-level throw during Astro's route collection reports as a route error rather than as the
 * verification failure it is.
 */
export function pageCards(): Record<string, Card> {
  const { bundle, verification } = loadFeed();
  /* The statute page's card counts what the corpus holds, on the same footing as every figure
     here: a card claiming five regimes that a sixth node makes wrong travels without the page
     beside it to correct it. `loadCorpus` is the same cached disk read the wiki's own cards use. */
  const corpus = loadCorpus();
  /* The bounds card's figures, off the same series the page draws. See the card below. */
  const bounds = census();
  const spans = regimes(corpus);
  const statutes = acts(corpus);
  const spanEnd =
    spans.length === 0
      ? 0
      : (lastOf(spans).to ?? Math.max(...statutes.map((a) => a.year)));
  const spanStart = spans.length === 0 ? 0 : firstOf(spans).from;
  /* The years `/history` draws, across its three sources, and the regimes they cross (#708). */
  const drawn = [bundle.history, bundle.meal_program, bundle.appropriations].flatMap((rows) =>
    rows.map((r) => r.fiscal_year),
  );
  const crossed = drawn.length === 0 ? [] : regimesCrossed(spans, Math.min(...drawn), Math.max(...drawn));
  const s = bundle.statewide;
  const fy = `FY${bundle.fiscal_year}`;
  const aid = millions(s.realized_aid_total).replace("+", "");
  /**
   * Two different guarantee shares, and they are not interchangeable.
   *
   * Half of Ohio's districts are on the guarantee and an eighth of the money is guarantee money —
   * 48% against 12% — because the districts held up by it are small. A bar has to answer for the
   * figure above it: under a dollar total, the share of dollars; under a count of districts, the
   * share of districts. Putting the district share under the dollar total, which this card did
   * first, states the wrong one of the two facts the site exists to separate.
   */
  const guaranteeOfAid = s.realized_aid_total > 0 ? s.guarantee_total / s.realized_aid_total : 0;
  const guaranteeOfDistricts = s.on_guarantee / s.districts;
  /* The biennium's two measures, off the same sums `/what-changed` prints. */
  const biennium = bundle.districts[0]!.biennium;
  const [total, foundation] = (["total", "foundation"] as const).map((m) => {
    const l = statewideLevels(bundle.districts, m);
    return l[2] - l[0];
  });

  /** The site's own card. What a link to anything unnamed here previews as. */
  const site: Card = {
    eyebrow: SITE,
    headline: "What Ohio pays its school districts",
    figure: aid,
    figureNote: `State foundation aid across ${s.districts} districts, ${fy}`,
    bar: {
      share: guaranteeOfAid,
      label: `${pct(guaranteeOfAid, 0)} of it from the guarantee`,
      tone: "guarantee",
    },
    // What the card is and when, in a reader's words. The contract version was a build label and
    // reached `/`, `/statewide` and the 404 through this one card (#593).
    meta: `${s.districts} districts · ${fy} model`,
  };

  return {
    /** `/og/default.png` renders this one too. Named so the table is the only copy. */
    site,

    statewide: {
      ...site,
      headline: "Every Ohio district's state aid, in one panel",
      figureNote: `State foundation aid across ${s.districts} districts, ${fy}`,
    },

    "what-changed": {
      eyebrow: SITE,
      headline: `What changed, FY${biennium.year_baseline} to FY${biennium.year_terminal}`,
      figure: millions(total!),
      figureNote: `Total state support across ${s.districts} districts, while foundation aid moved ${millions(foundation!)}`,
      meta: `FY${biennium.year_baseline} paid, FY${biennium.year_middle} and FY${biennium.year_terminal} models`,
    },

    districts: {
      eyebrow: SITE,
      headline: `All ${s.districts} school districts`,
      figure: count(s.on_guarantee),
      figureTone: "guarantee",
      figureNote: "are funded by the guarantee rather than by the formula",
      // A count of districts, so the share below it is a share of districts.
      bar: {
        share: guaranteeOfDistricts,
        label: `${pct(guaranteeOfDistricts, 0)} of the state's districts`,
        tone: "guarantee",
      },
      meta: `Aid per pupil, wealth and enrollment · ${fy}`,
    },

    counties: {
      eyebrow: SITE,
      headline: "Ohio's 88 counties",
      figure: "The gap within a county",
      figureNote:
        "Counties ranked by how far apart their richest and poorest district are on property wealth per pupil",
      meta: `${s.districts} districts, attributed to one county each · ${fy}`,
    },

    house: {
      eyebrow: SITE,
      headline: "The 99 Ohio House districts",
      figure: "Apportioned, not published",
      figureNote:
        "An estimate built from census blocks, since the state pays districts and no seat is a unit of account",
      meta: `${fy} model`,
    },

    senate: {
      eyebrow: SITE,
      headline: "The 33 Ohio Senate districts",
      figure: "Apportioned, not published",
      figureNote:
        "An estimate built from census blocks, since the state pays districts and no seat is a unit of account",
      meta: `${fy} model`,
    },

    compare: {
      eyebrow: SITE,
      headline: "Put two districts side by side",
      figure: `Any two of ${s.districts}`,
      figureNote: "Funding, guarantee status, property wealth, spending and outcomes, in one view",
      meta: `${fy} model`,
    },

    outcomes: {
      eyebrow: SITE,
      headline: "What the money is associated with",
      figure: "Funding against outcomes",
      figureNote:
        "The report-card measures beside the funding panel, with the confounders stated rather than controlled away",
      meta: `${s.districts} districts · ${fy}`,
    },

    legislation: {
      eyebrow: SITE,
      headline: "Ohio school funding in statute",
      figure: `${spans.length} formulas, ${spanEnd - spanStart + 1} years`,
      figureNote:
        "Every act from the 1851 constitutional duty to this biennium's budget, and what each did to the formula",
      /* The span, and not decoration: `og.spec.ts` requires a card carrying a number to name its
         year, because a card outlives the page it was shared from. This one's number *is* a span
         of years, so the two ends are the honest thing to date it with. */
      meta: `FY${spanStart}–FY${spanEnd} · ${statutes.length} acts, end to end`,
    },

    history: {
      eyebrow: SITE,
      headline: "How Ohio got here",
      figure: `${crossed.length} funding regimes`,
      figureNote:
        "From the foundation formula through DeRolph to the Fair School Funding Plan, and what each one paid",
      meta: `${fy} model, on a panel that starts in FY2009`,
    },

    scenario: {
      eyebrow: SITE,
      headline: "Move a lever, see who moves",
      figure: "Re-run the formula",
      figureNote: `Guarantee, base cost, minimum state share and phase-in, across all ${s.districts} districts`,
      meta: `Checked against ${verification.comparisons.length} reference scenarios · ${fy}`,
    },

    /*
     * The runner's other question, and the card says which half it is.
     *
     * The figure is the count of held districts rather than a dollar total on purpose: this page's
     * subject is who a change cannot reach, and the guarantee is the reason. It is the same
     * quantity `/districts` leads with, under a headline that says what it does to a lever.
     */
    reach: {
      eyebrow: SITE,
      headline: "Who a lever actually reaches",
      figure: count(s.on_guarantee),
      figureTone: "guarantee",
      figureNote:
        "districts are paid the guarantee, so a more generous formula reaches them last or not at all",
      bar: {
        share: guaranteeOfDistricts,
        label: `${pct(guaranteeOfDistricts, 0)} of the state's districts`,
        tone: "guarantee",
      },
      meta: `Every district against what its formula computes · ${fy}`,
    },

    /*
     * The one card whose figure counts the formula rather than the state.
     *
     * Derived from the series manifest for the reason every figure here is derived from the feed:
     * a card saying "38 bounds" that a thirty-ninth makes wrong travels onward without the page
     * beside it to correct it. The note carries the finding rather than the count, because the
     * count is already the figure and a card has one thing to say.
     */
    bounds: {
      eyebrow: SITE,
      headline: "Where the formula stops computing",
      figure: `${count(bounds.rows.length)} bounds`,
      figureNote: `floors, ceilings and clamps in the formula, ${
        bounds.cannotBind.length === 1 ? "one of which" : `${count(bounds.cannotBind.length)} of which`
      } can never bind for any district`,
      meta: `Ranked by the districts each one decides · ${fy}`,
    },

    method: {
      eyebrow: SITE,
      headline: "How these figures are made",
      figure: `${verification.comparisons.length} checkpoints`,
      figureNote:
        "The department's own model, re-run and required to match every reference scenario before the site builds",
      meta: `Method and verification · ${fy}`,
    },

    data: {
      eyebrow: SITE,
      headline: "Download the panel",
      figure: "JSON and CSV",
      figureNote:
        "The full feed, the formula inputs alone, and the district table — the same figures the pages are built from",
      meta: `Data downloads · ${fy}`,
    },

    // Only where the build publishes the index it is the card for (#718).
    ...(published().length > 0
      ? {
          explained: {
            eyebrow: SITE,
            headline: "The formula, one question at a time",
            figure: "Explained",
            figureNote: "The guarantee, the phase-in and the state share, in plain words",
            meta: "Every page ends at the evidence that proves it",
          },
        }
      : {}),

    wiki: {
      eyebrow: `${SITE} · Library`,
      headline: "How Ohio funds its public schools",
      figure: "The Library",
      figureNote:
        "The formula and its regimes, the property tax base, the litigation, and the programs around the formula",
      meta: "Every numeric claim one hop from its source",
    },

    sources: {
      eyebrow: `${SITE} · Library`,
      headline: "The sources behind the Library",
      figure: "Where each claim came from",
      figureNote:
        "Department publications, Legislative Service Commission analyses, district filings, and the litigation record",
      meta: "Provenance is one hop from every figure on this site",
    },
  };
}

/**
 * One card per Explained topic, keyed by slug: the question as the headline, and no figure.
 *
 * A topic's numbers come from its worked example, which is the whole page's computation, so the
 * card names the question and leaves the answer to the page. Read from the registry, so a topic
 * gets its card the day it is registered (#713).
 */
export function explainedCards(): Record<string, Card> {
  return Object.fromEntries(
    published().map((t) => [
      t.slug,
      {
        eyebrow: `${SITE} · Explained`,
        headline: clamp(t.question, 90),
        meta: `${t.group} · in plain words, with the math behind it`,
      },
    ]),
  );
}
