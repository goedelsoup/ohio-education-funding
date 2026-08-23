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

import { loadCorpus } from "../corpus.ts";
import { loadFeed } from "../feed.ts";
import { acts, regimes } from "../legislation.ts";
import { count, millions, pct } from "../format.ts";
import type { Card } from "./card.ts";

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
  const spans = regimes(corpus);
  const statutes = acts(corpus);
  const spanEnd =
    spans.length === 0
      ? 0
      : (spans[spans.length - 1]!.to ?? Math.max(...statutes.map((a) => a.year)));
  const spanStart = spans.length === 0 ? 0 : spans[0]!.from;
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
    meta: `Bundle contract ${bundle.contract_version} · ${fy} model`,
  };

  return {
    /** `/og/default.png` renders this one too. Named so the table is the only copy. */
    site,

    statewide: {
      ...site,
      headline: "Ohio school funding",
      figureNote: `State foundation aid across ${s.districts} districts, ${fy}`,
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
        "State aid distributed across legislative lines from census blocks — an estimate, since no seat is a unit of account",
      meta: `${fy} model`,
    },

    senate: {
      eyebrow: SITE,
      headline: "The 33 Ohio Senate districts",
      figure: "Apportioned, not published",
      figureNote:
        "State aid distributed across legislative lines from census blocks — an estimate, since no seat is a unit of account",
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
        "Every act from the 1851 constitutional duty to this biennium's budget, in the order it was signed, with what each one did to the formula",
      /* The span, and not decoration: `og.spec.ts` requires a card carrying a number to name its
         year, because a card outlives the page it was shared from. This one's number *is* a span
         of years, so the two ends are the honest thing to date it with. */
      meta: `FY${spanStart}–FY${spanEnd} · ${statutes.length} acts, end to end`,
    },

    history: {
      eyebrow: SITE,
      headline: "How Ohio got here",
      figure: "Four funding regimes",
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

    method: {
      eyebrow: SITE,
      headline: "How these figures are made",
      figure: `${verification.comparisons.length} checkpoints`,
      figureNote:
        "The department's own model, re-run and required to reproduce every reference scenario before the site may build",
      meta: `Bundle contract ${bundle.contract_version} · ${fy}`,
    },

    data: {
      eyebrow: SITE,
      headline: "Download the panel",
      figure: "JSON and CSV",
      figureNote:
        "The full feed, the formula inputs alone, and the district table — the same figures the pages are built from",
      meta: `Bundle contract ${bundle.contract_version} · ${fy}`,
    },

    wiki: {
      eyebrow: `${SITE} · Wiki`,
      headline: "How Ohio funds its public schools",
      figure: "The corpus",
      figureNote:
        "The formula and its regimes, the property tax base beside it, the litigation, and the programs that route money around it",
      meta: "Every numeric claim one hop from its source",
    },

    sources: {
      eyebrow: `${SITE} · Wiki`,
      headline: "The sources behind the corpus",
      figure: "Where each claim came from",
      figureNote:
        "Department publications, Legislative Service Commission analyses, district filings, and the litigation record",
      meta: "Provenance is one hop from every figure on this site",
    },
  };
}
