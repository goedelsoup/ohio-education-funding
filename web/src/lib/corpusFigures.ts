/**
 * The corpus-to-crate cross-check: does a `[verified — crates/X]` figure agree with `crates/X`?
 *
 * # The failure this exists to stop
 *
 * `[verified — crates/regime-diff]` is hand-typed text. Nothing in the repository relates it to
 * `crates/regime-diff`, so a correction's blast radius is set by whichever files the author
 * happened to open. The `recognized-valuation` correction reached three of its six carriers, and
 * two `formula-component` nodes went on publishing a **reversed sign** on the headline
 * distributional claim, live, under `[verified]`, while all 849 crate tests stayed green. See #120
 * and #131.
 *
 * `crates/figures.json` is the other end of that citation — each figure computed from the crate
 * that owns it, by `crates/figures`. A node binds a number in its prose to a key with a `figures:`
 * entry, and this module asserts the three-way agreement.
 *
 * # Why three ways and not one
 *
 * Each leg has failed here on its own:
 *
 * - **entry against manifest.** #128: seventeen numeric `[verified]` claims disagreeing with the
 *   source they cite. This is the leg the issue asks for.
 * - **phrase against entry.** A binding whose value is right and whose paragraph still says the
 *   old number changes nothing for a reader. The page is the product.
 * - **phrase against the field, verbatim.** The subtle one. #120's reversal was not a wrong
 *   numeral — it was the *same* two numerals attached to the opposite regimes. A numeral match
 *   passes that; a phrase match does not, because `316 districts would have done better under the
 *   charge-off` stops being a substring the moment the sentence is turned around.
 *
 * # What is deliberately not checkable here
 *
 * `revisions:` cannot be bound. A revision records what a node used to say, and
 * `.yidam/corpus/README.md` is explicit that the corpus is never rewritten to have always been
 * right — so a check that demanded a revision's prose track a moving figure would demand editing
 * the record of a correction. The `now:` half of a revision does restate a live figure, and the
 * node's body restates it too; the body is what binds.
 *
 * A direction carried by a word rather than a sign is also out of reach. "the median district is
 * $45 per pupil worse off" is bound to a figure named `median-shortfall-under-the-plan` for that
 * reason: the sign lives in the key, where a machine can see it, rather than in an adverb.
 */

import { readFileSync } from "node:fs";
import { existsSync } from "node:fs";
import { resolve } from "node:path";

import type { Node } from "./corpus.ts";

/**
 * The manifest contract this module reads.
 *
 * Checked before anything else, on the rule `feed.ts` follows: a consumer that does not recognise
 * the document refuses to run rather than comparing fields it may be misreading. The failure a
 * version check forecloses is the worst kind available to a gate — passing because it found
 * nothing to check. #125 catalogued sixteen of those.
 */
export const READS_CONTRACT = "1.0.0";

/** What a figure is measured in. Mirrors `figures::Unit`. */
export type Unit = "count" | "dollars" | "share" | "ratio";

/** One figure, computed from the crate that owns it. */
export interface ManifestFigure {
  key: string;
  owner: string;
  unit: Unit;
  value: number;
  label: string;
}

/** `crates/figures.json`, as written by `cargo run -p figures`. */
export interface Manifest {
  contract: string;
  figures: ManifestFigure[];
}

/**
 * Where `crates/figures` writes.
 *
 * Two candidates for the same reason `feed.ts` has two: the working directory is `web/` under
 * `pnpm test`, `pnpm build` and Playwright, and the repository root when something is run from
 * there.
 */
const CANDIDATES = ["../crates/figures.json", "crates/figures.json"];

let cached: Manifest | null = null;

/**
 * Read and check the manifest. Memoized.
 *
 * @throws if it is absent, unparseable, declares a contract this module does not read, or is
 * empty. An empty manifest is an error and not an empty check: every discrepancy below is
 * something a *binding* can be wrong about, so nothing here fails when there is nothing to
 * compare, and "the artefact did not build" would look exactly like "the corpus is correct".
 */
export function loadFigureManifest(): Manifest {
  if (cached) return cached;
  const path =
    CANDIDATES.map((candidate) => resolve(process.cwd(), candidate)).find((candidate) =>
      existsSync(candidate),
    ) ?? resolve(process.cwd(), CANDIDATES[0]!);

  let parsed: Manifest;
  try {
    parsed = JSON.parse(readFileSync(path, "utf8")) as Manifest;
  } catch (cause) {
    throw new Error(
      `Could not read the figure manifest at ${path}. Regenerate it with:\n` +
        `  cargo run --manifest-path crates/Cargo.toml -p figures > crates/figures.json\n\n` +
        `${String(cause)}`,
    );
  }

  if (parsed.contract !== READS_CONTRACT) {
    throw new Error(
      `This check reads figure manifest contract ${READS_CONTRACT}; ${path} declares ` +
        `${String(parsed.contract)}. Either crates/figures changed its fields and this module ` +
        `has not, or the manifest is from another tree.`,
    );
  }
  if (!Array.isArray(parsed.figures) || parsed.figures.length === 0) {
    throw new Error(
      `${path} carries no figures, so this check would pass against any corpus at all.`,
    );
  }
  cached = parsed;
  return parsed;
}

/** What a numeral in prose says, and how precisely it says it. */
export interface Numeral {
  /** The value, scaled into the figure's unit — `$5.1 million` is `5_100_000`. */
  value: number;
  /**
   * Half of the last written digit's place value.
   *
   * The precision of the writing sets the tolerance, which is the only rule here that needs no
   * per-figure judgement: `$5.1 million` claims the figure to the nearest hundred thousand and
   * `$674,561` claims it to the dollar, so each is held to what it claims. It is also the rule
   * that catches the class #153 was: `$0.97` where the subtraction gives `$0.96`, and `97.7%`
   * where the repository's own constant gives `97.1%`.
   */
  precision: number;
}

/**
 * Every numeral in a phrase, read as the given unit.
 *
 * Rejects rather than coerces on a unit mismatch. A `$` or a scale word on a `count`, or a bare
 * number where a `share` is expected, is not a near miss — it is a binding pointed at the wrong
 * figure, and reading it charitably would let `43 districts, $5.1 million withheld` satisfy a
 * count binding with its dollar figure.
 */
export function numerals(phrase: string, unit: Unit): Numeral[] {
  /*
   * The `\b` after the scale group is load-bearing. `m` is millions and `metres` is not, and
   * without a boundary to fail against, the optional group swallows the `m` of "65 more districts"
   * and reads sixty-five million. With it the group backtracks to empty and the numeral is read as
   * the sixty-five it is — which is the behaviour a *rejection* would not have given, because a
   * dropped numeral is a phrase that silently states nothing.
   */
  const pattern = /(\$)?(\d[\d,]*(?:\.\d+)?)(?:\s*(billion|million|thousand|bn|m|k))?\b(%)?/gi;
  const out: Numeral[] = [];
  for (const match of phrase.matchAll(pattern)) {
    const [, dollar, digits, scaleWord, percent] = match;
    const scale = scaleOf(scaleWord);
    if (!Number.isFinite(Number(digits!.replace(/,/g, "")))) continue;

    const bare = unit === "count" || unit === "ratio";
    if (bare && (dollar || percent || scale !== 0)) continue;
    if (unit === "share" && (dollar || scale !== 0)) continue;
    if (unit === "dollars" && percent) continue;

    /*
     * Shifted as text rather than multiplied, because a percent is a division by a hundred and
     * `8.2 / 100` is `0.08199999999999999`. Every tolerance here is wide enough to absorb that, and
     * it would still be the wrong number to report — a reader comparing the manifest against a
     * failure message should see the figure the author wrote. `Number("8.20e-2")` is `0.082`.
     */
    const shift = scale - (percent ? 2 : 0);
    const plain = digits!.replace(/,/g, "");
    const decimals = plain.includes(".") ? plain.split(".")[1]!.length : 0;
    out.push({
      value: Number(`${plain}e${shift}`),
      precision: Number(`1e${shift - decimals}`) / 2,
    });
  }
  return out;
}

/** The power of ten a scale word carries. */
function scaleOf(word: string | undefined): number {
  switch (word?.toLowerCase()) {
    case "billion":
    case "bn":
      return 9;
    case "million":
    case "m":
      return 6;
    case "thousand":
    case "k":
      return 3;
    default:
      return 0;
  }
}

/** Whitespace collapsed, so a re-wrapped YAML block scalar is the same prose. */
export function flatten(text: string): string {
  return text.replace(/\s+/g, " ").trim();
}

/** Every prose field of a node, by the name a `figures:` entry would use. */
export function proseFields(node: Node): Map<string, string> {
  const fields = new Map<string, string>();
  fields.set("summary", node.summary);
  fields.set("description", node.description);
  if (node.findings !== null) fields.set("findings", node.findings);
  for (const property of node.properties) fields.set(property.name, property.value);
  return fields;
}

/**
 * The claim tags a piece of prose carries, as `verified`/`inference`/`open` plus the justification.
 *
 * Deliberately not `badgeClaims` from `prose.ts`. That one runs over rendered HTML, skips code
 * spans, and returns markup; this needs the raw markdown a node was authored in, before any of
 * that. Two readers of one syntax is a real cost, and the alternative — rendering every node to
 * HTML to find out whether a paragraph says `[verified]` — is a worse one.
 */
export function claimTags(text: string): { tag: string; detail: string }[] {
  const pattern = /\[(verified|inference|open)((?:[^[\]]|\[[^[\]]*\])*)\]/g;
  return [...text.matchAll(pattern)].map((match) => ({
    tag: match[1]!,
    detail: match[2] ?? "",
  }));
}

/** What kind of thing is wrong. One per structural position the check can fail in. */
export type DiscrepancyKind =
  /** The node binds a key `crates/figures.json` does not carry. */
  | "unknown-key"
  /** The node's `value` is not what the crate computes. */
  | "value-disagrees"
  /** The node has no prose field by that name. */
  | "field-missing"
  /** The phrase is not in that field. */
  | "phrase-missing"
  /** No numeral in the phrase is the bound value, at the precision the phrase writes it to. */
  | "phrase-disagrees"
  /** More than one numeral in the phrase is, so the binding does not say which. */
  | "phrase-ambiguous"
  /** The bound field states no `[verified]` claim, so the figure is not offered as one. */
  | "field-unverified"
  /** The node binds a crate's figure and never cites that crate. */
  | "unattributed"
  /** A figure the manifest exports that no node binds. */
  | "uncited-figure";

/** One thing that is wrong, in the terms whoever has to fix it needs. */
export interface Discrepancy {
  /** The node's id, or `crates/figures.json` for a whole-manifest finding. */
  node: string;
  /** The figure key. */
  key: string;
  kind: DiscrepancyKind;
  /** What is wrong and what would resolve it. */
  message: string;
}

/**
 * Check every binding in the corpus against the manifest.
 *
 * Pure over its two arguments so that each position below can be broken on purpose in a test. #131
 * asks for exactly that, on the evidence of #125: sixteen checks in this repository were green
 * against the defect they were written for, and **every one was found by mutation rather than by
 * reading**. A gate whose failures are only hypothesised is a gate nobody has run.
 */
export function crossCheck(nodes: Node[], manifest: Manifest): Discrepancy[] {
  const byKey = new Map(manifest.figures.map((figure) => [figure.key, figure]));
  const found: Discrepancy[] = [];
  const bound = new Set<string>();

  for (const node of nodes) {
    if (node.figures.length === 0) continue;
    const fields = proseFields(node);
    const cited = citedCrates(node);

    for (const entry of node.figures) {
      const at = (kind: DiscrepancyKind, message: string) =>
        found.push({ node: node.id, key: entry.key, kind, message });

      const figure = byKey.get(entry.key);
      if (!figure) {
        at(
          "unknown-key",
          `binds "${entry.key}", which crates/figures.json does not carry. Either the key was ` +
            `renamed in crates/figures/src/lib.rs or the manifest is stale.`,
        );
        continue;
      }
      bound.add(figure.key);

      if (!agrees(entry.value, figure.value, figure.unit)) {
        at(
          "value-disagrees",
          `says ${figure.key} is ${entry.value}; ${figure.owner} computes ${figure.value}. ` +
            `The crate is the authority — the prose in "${entry.field}" has to move.`,
        );
        continue;
      }

      if (!cited.has(figure.owner)) {
        at(
          "unattributed",
          `binds a ${figure.owner} figure and cites ${figure.owner} in no claim tag. A figure ` +
            `checked against a crate the node never names is a check nobody reading the page ` +
            `could have anticipated.`,
        );
      }

      const prose = fields.get(entry.field);
      if (prose === undefined) {
        at(
          "field-missing",
          `binds it to a field "${entry.field}" this node does not have. Fields available: ` +
            `${[...fields.keys()].join(", ")}.`,
        );
        continue;
      }

      // `summary` is exempt, and only `summary`. It is the lead — capped at fifty words, barred
      // from carrying a markdown link, and substituted for the whole node by five call sites that
      // render no badges. A tag there would be a mark on a sentence nothing shows marks on. Every
      // other field states its own confidence, and a figure offered for checking against a crate
      // without saying it is verified is a figure a reader cannot grade.
      if (entry.field !== "summary" && claimTags(prose).every((claim) => claim.tag !== "verified")) {
        at(
          "field-unverified",
          `binds a figure into "${entry.field}", which states no [verified] claim. A figure ` +
            `checked against a crate is a verified figure; say so where it is written.`,
        );
      }

      if (!flatten(prose).includes(flatten(entry.as_written))) {
        at(
          "phrase-missing",
          `quotes "${entry.as_written}", which is not in "${entry.field}". The paragraph was ` +
            `edited and the binding was not, or the other way round.`,
        );
        continue;
      }

      const faithful = numerals(entry.as_written, figure.unit).filter(
        (numeral) => Math.abs(numeral.value - entry.value) <= numeral.precision,
      );
      if (faithful.length === 0) {
        at(
          "phrase-disagrees",
          `quotes "${entry.as_written}", which states no number equal to ${entry.value} at the ` +
            `precision it is written to. The phrase and the value it is bound to have come apart.`,
        );
      } else if (faithful.length > 1) {
        at(
          "phrase-ambiguous",
          `quotes "${entry.as_written}", where ${faithful.length} numbers are ${entry.value}. ` +
            `Quote a shorter phrase, so the binding says which one it means.`,
        );
      }
    }
  }

  for (const figure of manifest.figures) {
    if (bound.has(figure.key)) continue;
    found.push({
      node: "crates/figures.json",
      key: figure.key,
      kind: "uncited-figure",
      message:
        `${figure.owner} exports "${figure.key}" and no node binds it. A figure nothing cites ` +
        `is a computation nothing checks — bind it, or drop it from figures::FIGURES.`,
    });
  }

  return found;
}

/**
 * Whether a node's stated value is the crate's.
 *
 * Exact for a count. Otherwise held to the precision the node writes it to, which is the same rule
 * {@link Numeral} applies to prose and for the same reason: a node states `5110049.66` where the
 * crate holds `5110049.659999998`, and the difference is the node declining to claim fractions of
 * a cent rather than the node being wrong.
 *
 * A fixed relative epsilon was the obvious alternative and is the wrong one. At the magnitude of
 * `targeted-assistance-median-weighted-wealth` a part-per-million epsilon is nearly four hundred
 * dollars of slack, so a node could restate that figure with its last six digits wrong and pass.
 * The precision a claim is written to is the only bound that scales with the claim.
 */
function agrees(stated: number, computed: number, unit: Unit): boolean {
  if (unit === "count") return stated === computed;
  return Math.abs(stated - computed) <= halfStep(stated);
}

/**
 * Half the place value of a number's last written digit.
 *
 * Exponent notation falls back to a part-per-billion relative bound: `1e-7` says nothing about how
 * precisely it was meant, and refusing to guess is better than reading it as exact.
 */
function halfStep(value: number): number {
  const written = String(value);
  if (written.includes("e") || written.includes("E")) {
    return Math.max(Math.abs(value), 1) * 1e-9;
  }
  const decimals = written.includes(".") ? written.split(".")[1]!.length : 0;
  return 10 ** -decimals / 2;
}

/**
 * Every `crates/…` directory this node names inside a claim tag.
 *
 * Inside a tag rather than anywhere in the prose. A crate mentioned in running text is a pointer;
 * a crate named in `[verified — …]` is the node saying *this is what settles it*, which is the
 * assertion the whole mechanism is about. Scans `linkText`, which the loader assembles from every
 * place a node writes prose — including `revisions:`, since a citation that survives only in a
 * withdrawal is still the node naming its authority.
 */
function citedCrates(node: Node): Set<string> {
  const prose = [node.summary, node.description, node.linkText].join("\n\n");
  const crates = new Set<string>();
  for (const claim of claimTags(prose)) {
    for (const match of claim.detail.matchAll(/crates\/([a-z0-9-]+)/g)) {
      crates.add(`crates/${match[1]}`);
    }
  }
  return crates;
}
