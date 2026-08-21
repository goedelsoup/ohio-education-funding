/**
 * Fifty years of Ohio school funding as statute, on one axis.
 *
 * # What this page can show that `/wiki/legislation` cannot
 *
 * The class page lists sixteen acts with their summaries, and that is the right shape for "what
 * is in this class". It cannot show succession. An act's page says it continues the Bridge
 * Formula; the class page next to it says another one establishes the Fair School Funding Plan;
 * nothing puts the two in an order, and nothing anywhere says how long either lasted.
 *
 * The corpus already holds the answer and has never rendered it. Every `funding-regime` node
 * carries `effective_from` and `effective_to` as fiscal-year labels, and the five of them turn out
 * to **partition fifty years with no gap and no overlap** — Equal Yield, Foundation Base Cost,
 * Evidence-Based Model, Bridge, Fair School Funding Plan, end to end. Every `fiscal-period` node
 * carries `start` and `end` as ISO dates. Every act carries `signed`, and thirteen of the sixteen
 * point at the biennium they appropriate for.
 *
 * So the spine is measured rather than drawn: the bands are the corpus's own spans, the acts sit
 * at their own signed dates, and where a regime lasted one biennium the page says so because the
 * arithmetic says so.
 *
 * # Nothing here restates a node's prose
 *
 * The temptation on a page like this is to summarise sixteen acts, which would put a second,
 * unversioned copy of the corpus in the web layer — the failure the four prose genres exist to
 * prevent. What is rendered is what the *edges* say: which regime an act established, continued,
 * amended or corrected, which biennium it funded, and which earlier act it amends. A reader who
 * wants what an act did opens the act.
 *
 * # Every year on this page is read
 *
 * There is not one year literal in this file. `tests/unit/yearLiterals.spec.ts` would allow a few
 * with a reason, and none is needed: the bands come from `effective_from` / `effective_to`, the
 * act dates from `signed`, the biennia from `start` / `end`.
 */

import type { Corpus, Node } from "./corpus.ts";
import { escapeHtml } from "./format.ts";
import * as routes from "./routes.ts";
import { anchor } from "./section.ts";

/** A fiscal-year label as the corpus writes it — `FY2012`, or `current` for a regime still running. */
export function fiscalYear(label: string): number | null {
  const match = label.trim().match(/^FY(\d{4})$/);
  return match ? Number.parseInt(match[1]!, 10) : null;
}

/** A property, or the empty string. Nodes in one class are uniform, but nothing enforces it. */
function prop(node: Node, name: string): string {
  return node.properties.find((p) => p.name === name)?.value.trim() ?? "";
}

/** The four-digit year of an ISO date. */
function isoYear(date: string): number | null {
  const year = Number.parseInt(date.slice(0, 4), 10);
  return Number.isFinite(year) ? year : null;
}

export interface Regime {
  id: string;
  name: string;
  href: string;
  /** First fiscal year. */
  from: number;
  /** Last fiscal year, or `null` where the regime is the one still running. */
  to: number | null;
  status: string;
  /** The act that put it in place, where the corpus holds one. Two of the five predate the class. */
  establishedBy: Act | null;
}

/** What an act did to a regime, in the corpus's own vocabulary. */
export type Action = "establishes" | "continues" | "amends" | "corrects";

const ACTIONS: Action[] = ["establishes", "continues", "amends", "corrects"];

export interface Act {
  id: string;
  href: string;
  /** `Am. Sub. H.B. 110`. The designation verbatim — `Am. Sub.` is part of the bill's name. */
  designation: string;
  signed: string;
  year: number;
  assembly: string;
  /** What it did to which regime, where it did anything. */
  action: { verb: Action; regime: string; href: string } | null;
  /** The biennium or year it appropriates for. Absent on the three acts that are not budgets. */
  funds: { label: string; href: string; start: string; end: string } | null;
  /** The earlier acts it amends, newest first. The chain, as the corpus states it. */
  amends: { label: string; href: string }[];
}

const REGIME = "funding-regime/";

/**
 * The acts, oldest first.
 *
 * Sorted on `signed` as a string, which is correct because they are ISO dates and lexical order
 * is chronological order for those. Sorting on the parsed year would tie H.B. 110 and H.B. 583's
 * predecessors within a General Assembly.
 */
export function acts(corpus: Corpus): Act[] {
  const nodes = corpus.byClass.get("legislation")?.nodes ?? [];
  const byId = new Map(nodes.map((n) => [n.id, n]));

  const built = nodes.map((node): Act => {
    const signed = prop(node, "signed");
    const year = isoYear(signed);
    if (year == null) {
      throw new Error(`${node.id} has no parseable \`signed\` date; the timeline is built on it`);
    }

    const regimeEdge = node.out.find(
      (e) => e.id?.startsWith(REGIME) && ACTIONS.includes(e.relationship as Action),
    );
    const fundsEdge = node.out.find((e) => e.relationship === "appropriates-for" && e.id);
    const period = fundsEdge?.id ? corpus.byId.get(fundsEdge.id) : undefined;

    const act: Act = {
      id: node.id,
      href: routes.wikiNode(node.className, node.name),
      designation: prop(node, "designation") || node.label,
      signed,
      year,
      assembly: prop(node, "general_assembly"),
      action: regimeEdge
        ? {
            verb: regimeEdge.relationship as Action,
            regime: regimeEdge.label,
            href: regimeEdge.href,
          }
        : null,
      funds:
        period != null
          ? {
              label: prop(period, "label") || period.label,
              href: routes.wikiNode(period.className, period.name),
              start: prop(period, "start"),
              end: prop(period, "end"),
            }
          : null,
      amends: node.out
        .filter((e) => e.relationship === "amends" && e.id != null && byId.has(e.id))
        .map((e) => {
          const target = byId.get(e.id!)!;
          return { label: prop(target, "designation") || target.label, href: e.href };
        }),
    };
    return act;
  });

  return built.sort((a, b) => a.signed.localeCompare(b.signed));
}

/**
 * The regimes, oldest first, each with the act that established it where the corpus holds one.
 *
 * Two of the five have no establishing act and will not get one: `Equal Yield` began in FY1976 and
 * `Foundation Base Cost` in FY1992, and the legislation class reaches back to H.B. 920 (1976) and
 * then to the Constitution. Rendering an empty cell there would read as missing data. It is not —
 * it is the edge of what this corpus covers, and the page says which.
 */
export function regimes(corpus: Corpus): Regime[] {
  const all = acts(corpus);
  const establishedBy = new Map<string, Act>();
  for (const act of all) {
    if (act.action?.verb === "establishes") establishedBy.set(act.action.href, act);
  }

  const nodes = corpus.byClass.get("funding-regime")?.nodes ?? [];
  const built = nodes.flatMap((node): Regime[] => {
    const from = fiscalYear(prop(node, "effective_from"));
    if (from == null) return [];
    const href = routes.wikiNode(node.className, node.name);
    return [
      {
        id: node.id,
        name: prop(node, "name") || node.label,
        href,
        from,
        to: fiscalYear(prop(node, "effective_to")),
        status: prop(node, "status"),
        establishedBy: establishedBy.get(href) ?? null,
      },
    ];
  });

  return built.sort((a, b) => a.from - b.from);
}

/**
 * Whether the regimes actually tile the years, and where they do not.
 *
 * The claim the band chart makes by drawing them end to end. It holds today, and it is exactly the
 * kind of claim that stops holding without anyone noticing: a regime added with a boundary off by
 * one year draws a chart with an invisible seam, and a `effective_to` edited to overlap draws two
 * regimes in force at once — which is a statement about Ohio, made by a stylesheet.
 *
 * Only the regime still running may have no end. A second open-ended regime is an overlap that
 * reaches the present.
 */
export function succession(spans: Regime[]): string[] {
  const problems: string[] = [];
  spans.forEach((regime, index) => {
    const next = spans[index + 1];
    if (regime.to == null) {
      if (next) problems.push(`${regime.name} has no end but ${next.name} follows it`);
      return;
    }
    if (regime.to < regime.from) problems.push(`${regime.name} ends before it begins`);
    if (!next) return;
    if (next.from === regime.to + 1) return;
    problems.push(
      next.from > regime.to + 1
        ? `nothing runs between ${regime.name} and ${next.name}`
        : `${regime.name} and ${next.name} overlap`,
    );
  });
  return problems;
}

/** `FY2012–FY2021`, or `FY2022 onward` for the one still running. En dash, not a hyphen. */
function span(regime: Regime): string {
  return regime.to == null ? `FY${regime.from} onward` : `FY${regime.from}–FY${regime.to}`;
}

/** How many biennia a regime lasted, which is what makes a two-year regime visible as one. */
function biennia(regime: Regime, latest: number): number {
  const end = regime.to ?? latest;
  return Math.max(1, Math.round((end - regime.from + 1) / 2));
}

const VERB: Record<Action, string> = {
  establishes: "establishes",
  continues: "continues",
  amends: "amends",
  corrects: "corrects",
};

function link(href: string, text: string): string {
  return `<a href="${escapeHtml(href)}">${escapeHtml(text)}</a>`;
}

/**
 * The page.
 *
 * Three cards, and the order is the argument: what was in force, what put it there, and the two
 * instruments that were in force under all of it.
 */
export function renderTimeline(corpus: Corpus): string {
  const spans = regimes(corpus);
  const all = acts(corpus);
  if (spans.length === 0 || all.length === 0) return "";

  const first = spans[0]!;
  const current = spans[spans.length - 1]!;
  const latest = Math.max(...all.map((a) => a.year));
  const covered = (current.to ?? latest) - first.from + 1;

  /*
   * The bar each regime gets, as an offset and a width in percent of the covered span.
   *
   * The first shape of this was a single stacked strip with the names inside the bands, and it
   * failed on the fact it existed to show: the Evidence-Based Model lasted one biennium, so its
   * band was forty pixels wide and read `Ev…`. A chart whose narrowest element is the one worth
   * looking at cannot put the label inside the element. Here the name is a table cell at full
   * width and the bar sits beside it, so the proportion is visible and legible at once.
   */
  const bar = (regime: Regime): string => {
    const end = regime.to ?? latest;
    const offset = ((regime.from - first.from) / covered) * 100;
    const width = ((end - regime.from + 1) / covered) * 100;
    return `<span class="span-bar"><i style="margin-left:${offset.toFixed(2)}%;width:${width.toFixed(2)}%"></i></span>`;
  };

  const regimeRows = spans
    .map(
      (regime) => `
        <tr>
          <th>${link(regime.href, regime.name)}</th>
          <td class="span">${bar(regime)}</td>
          <td class="year">${escapeHtml(span(regime))}</td>
          <td>${biennia(regime, latest)}</td>
          <td>${escapeHtml(regime.status.replace(/-/g, " "))}</td>
          <td>${
            regime.establishedBy
              ? link(regime.establishedBy.href, regime.establishedBy.designation)
              : `<span class="caveat-inline">older than this corpus reaches</span>`
          }</td>
        </tr>`,
    )
    .join("");

  const actRows = all
    .map((act) => {
      const amends =
        act.amends.length === 0
          ? ""
          : `<span class="amends">amends ${act.amends
              .map((a) => link(a.href, a.label))
              .join(", ")}</span>`;
      return `
        <tr>
          <td class="year">${act.year}</td>
          <th>${link(act.href, act.designation)}${amends}</th>
          <td>${escapeHtml(act.assembly)}</td>
          <td>${
            act.action
              ? `${VERB[act.action.verb]} ${link(act.action.href, act.action.regime)}`
              : `<span class="caveat-inline">no formula edge</span>`
          }</td>
          <td>${
            act.funds
              ? link(act.funds.href, act.funds.label)
              : `<span class="caveat-inline">not a budget act</span>`
          }</td>
        </tr>`;
    })
    .join("");

  const standing = all.filter((act) => act.funds == null && act.amends.length === 0);

  return `
    <div class="card" id="regimes" data-part="regimes">
      <h2>${anchor("regimes")}What was in force, and for how long</h2>
      <p class="note">${spans.length} formulas across ${covered} fiscal years, end to end. The
        spans are each regime's own <code>effective_from</code> and <code>effective_to</code>; that
        they tile the years with no gap and no overlap is checked rather than drawn.</p>

      <div class="scroll">
        <table class="prose spans">
          <thead>
            <tr><th>Formula</th><th>Span</th><th>Years</th><th>Biennia</th><th>Status</th><th>Established by</th></tr>
          </thead>
          <tbody>${regimeRows}</tbody>
        </table>
      </div>
      <p class="note">Two of the ${spans.length} have no establishing act here and will not get
        one: they begin before the oldest instrument this corpus holds. That is the edge of the
        collection rather than a hole in it.</p>
    </div>

    <div class="card" id="acts" data-part="acts">
      <h2>${anchor("acts")}Every act, in the order it was signed</h2>
      <p class="note">What each one did to the formula and which biennium it paid for, from the
        edges the corpus states. What an act <em>says</em> is on its own page — restating it here
        would put a second copy of the corpus in the site.</p>
      <div class="scroll">
        <table class="prose acts">
          <thead>
            <tr><th>Signed</th><th>Act</th><th>General Assembly</th><th>Formula</th><th>Paid for</th></tr>
          </thead>
          <tbody>${actRows}</tbody>
        </table>
      </div>
    </div>

    ${
      standing.length === 0
        ? ""
        : `<div class="card" id="standing" data-part="standing">
      <h2>${anchor("standing")}The instruments that never expired</h2>
      <p class="note">${standing.length} of the ${all.length}: they appropriate nothing, amend
        nothing, and were in force under every formula in the band above. A budget act is repealed
        by the next budget act. These were not.</p>
      <div class="scroll">
        <table class="prose">
          <tbody>
            ${standing
              .map(
                (act) => `<tr>
                  <td class="year">${act.year}</td>
                  <th>${link(act.href, act.designation)}</th>
                  <td>${escapeHtml(corpus.byId.get(act.id)?.summary ?? "")}</td>
                </tr>`,
              )
              .join("")}
          </tbody>
        </table>
      </div>
    </div>`
    }`;
}
