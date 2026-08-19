/**
 * Where things live.
 *
 * One module so a route is renamed in one place rather than in the 40-odd templates that link to
 * it, and so the build-time link checker in `tests/` has something to enumerate against.
 *
 * Every path is absolute from the site root and carries no trailing slash. `build.format` is
 * `"file"`, so `/district/043786` is served by `district/043786.html` and adding a slash would
 * 404 on the static host.
 */

/** The district a route is about. IRN, always — 28 district names in the feed are not unique. */
export const district = (irn: string): string => `/district/${irn}`;
export const districtOutcome = (irn: string): string => `/district/${irn}/outcome`;
export const districtScenario = (irn: string): string => `/district/${irn}/scenario`;
export const districtFinances = (irn: string): string => `/district/${irn}/finances`;
export const districtTaxes = (irn: string): string => `/district/${irn}/taxes`;

/**
 * The addressable sections of every route.
 *
 * # Why there is a vocabulary rather than a fragment written where it is needed
 *
 * Nine entry surfaces deposit every reader at byte zero of the same 50,000-byte document, and for
 * most of them the question they arrived with has an answer four cards down. Before this there was
 * exactly one `id` in the whole rendering layer — `prose.ts`'s correction blockquote — so there was
 * nothing to send them to.
 *
 * Every name below is a `data-part` on a card or a `data-program` on a sub-table, so a section is
 * never invented here: `SECTIONS` names what the markup already distinguishes, and
 * `tests/e2e` asserts that every one of these strings resolves to an element with that `id` in the
 * built page. That is the check the `routes.parameter("state-share-percentage")` 404 got past —
 * a fragment written inline in a template is enumerated by nothing, and a missing one fails
 * silently rather than 404ing, which is worse.
 *
 * # Why it is grouped, and why it now covers everything
 *
 * It held 21 names, all of them on the district routes, and the rest of the site had none: 80 of
 * the 111 cards carried no `id` at all, so `/method`, `/history`, `/outcomes` and every wiki page
 * were one address each. It was also barely used — two links in the repository ever named a
 * section — for the reason `section.ts` gives: an address a reader cannot see is an address nobody
 * asks for. Both halves are fixed together, because either alone does nothing.
 *
 * The grouping is by route family and is documentation rather than enforcement: an `id` has only
 * to be unique within the page that carries it, which is why `roster` and `national` each appear
 * under more than one family meaning the thing that family means by it. What the grouping buys is
 * that a reader of this file can see which page a name belongs to without opening the template,
 * and `check-dist-links.ts` can assert that the built site carries no section id this table does
 * not list.
 */
export const SECTIONS = {
  /** The five `/district/[irn]` routes. */
  district: {
    aidSource: "aid-source",
    enrollment: "enrollment",
    actuals: "actuals",
    baseCost: "base-cost",
    categoricals: "categoricals",
    supplements: "supplements",
    position: "position",
    national: "national",
    specialEducation: "special-education",
    targetedAssistance: "targeted-assistance",
    dpia: "dpia",
    gifted: "gifted",
    careerTechnical: "career-technical",
    englishLearners: "english-learners",
    transportation: "transportation",
    preschool: "preschool",
    taxBase: "tax-base",
    valuationChange: "valuation-change",
    millage: "millage",
    denominators: "denominators",
    chargeOff: "charge-off",
    taxEffort: "tax-effort",
    spendingByFunction: "spending-by-function",
    federalShare: "federal-share",
    outcomes: "outcomes",
    comparablePoverty: "comparable-poverty",
    levers: "levers",
    /* The states a route can be in rather than sections of it — no filing, no SD-1 row, no
       JavaScript, no district. They are addressed on the same terms as everything else because
       the alternative is a card the ratchet has to be taught to skip. */
    not: "not",
    noFiling: "no-filing",
    notPublished: "not-published",
    needsScript: "needs-script",
  },

  /** The statewide view at `/`. */
  statewide: {
    finances: "finances",
    guarantee: "guarantee",
    wealthOffset: "wealth-offset",
    twoFloors: "two-floors",
    national: "national",
    findADistrict: "find-a-district",
    threeQuestions: "three-questions",
  },

  /** `/history` — the Census panel, and the appropriation acts beside it. */
  history: {
    revenueMix: "revenue-mix",
    equityGap: "equity-gap",
    appropriations: "appropriations",
    lineOrigins: "line-origins",
    mealProgram: "meal-program",
    whatThisIs: "what-this-is",
    noPanel: "no-panel",
  },

  /** `/outcomes`. */
  outcomes: {
    povertyAndPerformance: "poverty-and-performance",
    guaranteeTrap: "guarantee-trap",
    twoDenominators: "two-denominators",
    limits: "limits",
    noData: "no-outcome-data",
  },

  /** `/counties` and `/county/[slug]`. */
  county: {
    peerGroup: "peer-group",
    roster: "roster",
    spread: "spread",
  },

  /** `/house`, `/senate`, and `/[chamber]/[number]`. */
  chamber: {
    estimates: "estimates",
    roster: "roster",
    apportioned: "apportioned",
    members: "members",
  },

  /** `/method`. */
  method: {
    computedTwice: "computed-twice",
    pupilCounts: "pupil-counts",
    modelAndRecord: "model-and-record",
    baseCost: "base-cost",
    reductionFactors: "reduction-factors",
    forecastRange: "forecast-range",
    provenance: "provenance",
    notHere: "not-here",
    checkingItYourself: "checking-it-yourself",
  },

  /** `/data`. */
  data: {
    downloads: "downloads",
    checkpoints: "checkpoints",
    terms: "terms",
  },

  /** `/scenario`, and the cards its runner writes into the page. */
  scenario: {
    heldFixed: "held-fixed",
    needsScript: "needs-script",
    levers: "levers",
    currentLaw: "current-law",
    projection: "projection",
    distribution: "distribution",
    mostAffected: "most-affected",
    movedHere: "moved-here",
    movedElsewhere: "moved-elsewhere",
    movedUnderneath: "moved-underneath",
    unknownDistrict: "unknown-district",
    /* Written by `scripts/scenario.ts` rather than by the build, into a div the built page
       carries empty. Addressed on the same terms so the runtime half of this page and the
       server-rendered half do not diverge in what a reader can link to. */
    disabled: "disabled",
    projectionDisabled: "projection-disabled",
    panelUnreachable: "panel-unreachable",
  },

  /** `/wiki` and its four sub-families. */
  wiki: {
    whatThisIs: "what-this-is",
    classes: "classes",
    sources: "sources",
    decisions: "decisions",
    description: "description",
    nodes: "nodes",
    properties: "properties",
    relationships: "relationships",
    links: "links",
    findings: "findings",
    district: "district",
    revisions: "revisions",
    why: "why",
    record: "record",
    records: "records",
    citedBy: "cited-by",
    summary: "summary",
    corrections: "corrections",
    /* A decision record renders one card per field it carries, addressed by the field's own name.
       These are `DECISION_SECTIONS` in `corpus.ts` — restated rather than imported because that
       module reads `.yidam/` off disk and this one must stay a table of strings. The two agreeing
       is asserted in `tests/unit/links.spec.ts`. */
    context: "context",
    decision: "decision",
    rationale: "rationale",
    consequences: "consequences",
    amendment: "amendment",
    alternatives: "alternatives",
  },

  /** `/search`, `/compare` and `/404` — the three routes that are not about anything. */
  tools: {
    query: "query",
    pick: "pick",
    needsScript: "needs-script",
    suggest: "suggest",
    whereToGo: "where-to-go",
    /* Written by `scripts/search.ts` and `scripts/compare.ts` into a div the built page carries
       empty, on the same terms as the scenario runner's cards above. */
    results: "results",
    noMatches: "no-matches",
    indexUnreachable: "index-unreachable",
    comparison: "comparison",
    panelUnreachable: "panel-unreachable",
  },
} as const;

/**
 * Every section name on the site, across all families.
 *
 * Written as a mapped type indexed by its own keys rather than the shorter
 * `(typeof SECTIONS)[keyof typeof SECTIONS][keyof …]`: `keyof` over a union of object types is the
 * *intersection* of their keys, which for these ten families is empty, so the shorter spelling
 * silently evaluates to `never` and every call to `at()` stops type-checking.
 */
export type Section = {
  [Route in keyof typeof SECTIONS]: (typeof SECTIONS)[Route][keyof (typeof SECTIONS)[Route]];
}[keyof typeof SECTIONS];

/** Flat, for the checks that ask whether a given id is in the vocabulary at all. */
export const SECTION_NAMES: ReadonlySet<string> = new Set(
  Object.values(SECTIONS).flatMap((family) => Object.values(family)),
);

/**
 * A section of a route.
 *
 * Takes a route rather than an IRN so the caller says which of the five it means, and the section
 * name is typed so a rename is a compile error rather than a fragment that silently lands the
 * reader at the top of the right page.
 */
export const at = (route: string, id: Section): string => `${route}#${id}`;

/**
 * A county, by its slug.
 *
 * Counties are a peer group rather than a boundary here — the department attributes each district
 * to one county and school district lines cross county lines freely — so the route is
 * `/county/…` singular, naming the page's subject rather than claiming an aggregate.
 */
export const county = (slug: string): string => `/county/${slug}`;

/**
 * An Ohio House district, by its number.
 *
 * Zero-padded to three characters, as the Census files number them: `/house/024`. Unpadded would
 * make `/house/24` and `/house/024` two paths for one district on a static host.
 */
export const houseDistrict = (number: string): string => `/house/${number}`;
/** And the Senate, on the same shape. */
export const senateDistrict = (number: string): string => `/senate/${number}`;
/** Either chamber's index. */
export const chamberIndex = (chamber: "house" | "senate"): string => `/${chamber}`;

/**
 * The path a built page is *served* at, from the path it was written to.
 *
 * `build.format` is `"file"`, so Astro writes `/district/043786` to `district/043786.html` — and
 * during the build `Astro.url.pathname` reports the second of those, because that is the file being
 * produced. Anything a page says about its own address therefore has to be put back: the canonical
 * link and `og:url` both name a URL a reader can visit, and `…/043786.html` is not one.
 *
 * This is not hypothetical tidying. The canonical link shipped as `…/district/043786.html` while
 * `sitemap-0.xml` listed `…/district/043786`, which is one page telling a crawler two different
 * things about which of its two addresses is the real one.
 */
export function canonicalPath(pathname: string): string {
  // `/index.html` is the site root, not a page called "index" — and it is the only page whose
  // served path is shorter than its name rather than the same minus an extension.
  if (pathname === "/index.html" || pathname === "/index") return "/";
  const stripped = pathname.endsWith(".html") ? pathname.slice(0, -".html".length) : pathname;
  // A trailing slash 404s on the static host, so it is never correct to leave one on.
  return stripped.length > 1 && stripped.endsWith("/") ? stripped.slice(0, -1) : stripped;
}

/**
 * The preview cards, which are routes like any other.
 *
 * Under `/og/` and mirroring the page routes below them, so the card for `/county/allen` is at
 * `/og/county/allen.png` and neither can be renamed without the other being obviously wrong.
 *
 * A district has five pages and one card. `/district/X/finances` and `/district/X/taxes` are the
 * same district and the preview says so; what distinguishes them in a feed is `og:title`, which
 * carries the page's own title. Rendering five near-identical images per district would be 3,045
 * of them rather than 609, for a difference no reader would see.
 */
export const og = {
  /** Any page that has not been given a card of its own. */
  default: (): string => "/og/default.png",
  district: (irn: string): string => `/og/district/${irn}.png`,
  county: (slug: string): string => `/og/county/${slug}.png`,
  chamberDistrict: (chamber: "house" | "senate", number: string): string =>
    `/og/${chamber}/${number}.png`,
  wikiClass: (className: string): string => `/og/wiki/${className}.png`,
  /**
   * A node's card — and a source's, since `source` is the fourteenth pseudo-class here exactly as
   * it is in the page routes above.
   */
  wikiNode: (className: string, node: string): string => `/og/wiki/${className}/${node}.png`,
  wikiSource: (slug: string): string => `/og/wiki/source/${slug}.png`,
  wikiDecision: (slug: string): string => `/og/wiki/decision/${slug}.png`,
  /** A top-level page — `/`, `/method`, `/data` — keyed by the slug in `src/lib/og/pages.ts`. */
  page: (slug: string): string => `/og/page/${slug}.png`,
} as const;

/** A corpus node, by its class and file stem. */
export const wikiNode = (className: string, node: string): string =>
  `/wiki/${className}/${node}`;
/** A corpus class index, which doubles as the rendered ontology for that class. */
export const wikiClass = (className: string): string => `/wiki/${className}`;
/**
 * A catalog entry.
 *
 * Under `/wiki/` rather than at the top level: the sources are part of the same graph as the
 * nodes that cite them, and provenance in this domain is one hop from every numeric claim. It is
 * a fourteenth pseudo-class, not a separate section of the site.
 */
export const wikiSource = (slug: string): string => `/wiki/source/${slug}`;

/**
 * A decision record.
 *
 * `decision`, singular, for the same reason `source` is: the segment names what one page is, not
 * the directory the file came out of. `/wiki/decisions/…` was the shape produced by an old bug
 * that read the `decisions/` subtree as an ontology class, and it is worth never building again.
 */
export const wikiDecision = (slug: string): string => `/wiki/decision/${slug}`;

/** Two districts side by side. Query parameters, because the pair is chosen by the reader. */
export function compare(a?: string, b?: string): string {
  if (!a || !b) return "/compare";
  return `/compare?a=${encodeURIComponent(a)}&b=${encodeURIComponent(b)}`;
}

/**
 * A metric, parameter, or other corpus concept referenced from a data page.
 *
 * The link out from a figure to what the figure *means* is the reason the wiki is worth
 * building; a glossary nobody arrives at from the number they were reading is a document museum.
 */
export const metric = (node: string): string => wikiNode("metric", node);
export const parameter = (node: string): string => wikiNode("parameter", node);
