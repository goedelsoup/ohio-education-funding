/**
 * Split one node's `description:` into description / findings, textually.
 *
 * Textual, not a YAML round-trip: every block scalar in this corpus is hand-wrapped at column 95
 * and a dumper would reflow all of them, burying the change in a diff nobody could review.
 *
 * The partition is given per node as paragraph indices. The judgement of which paragraph is the
 * subject and which is a computed finding is not something a heuristic should make — a finding
 * misfiled as subject reads as though Ohio publishes it.
 *
 * Usage: node partition.mjs <plan.json>
 *   { "<class>/<node>": { "findings": [4,5,6], "drop": [10], "revisions": [{...}] } }
 * Paragraph indices not listed stay in `description`, in their original order.
 *
 * `drop` removes a paragraph outright, for the source text of a `revisions` entry. Without it the
 * withdrawal ends up in both places — stated as prose in the body *and* as a structured entry —
 * which is worse than either alone, because the body copy is the half a reader meets first and it
 * is the half that reads as current.
 */
import { readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";

import YAML from "yaml";

const CORPUS = "/Users/cparent/Code/goedelsoup/ohio-education-funding/main/.yidam/corpus";

/** Every line of the paragraph is quoted — the whole thing is one block quotation. */
const QUOTE = /^(?: *>.*(?:\n|$))+$/;
/** A line-leading list marker. Group 1 is `.` for an ordered item, the bullet character else. */
const MARKER = /^ *(?:[-*](?= )|\d+(\.)(?= ))/m;

/** Fold one field's text to the corpus's 95-column wrap at a given indent. */
function fold(text, indent) {
  return text
    .split("\n\n")
    .map((para) => {
      // A block indented four or more spaces is an ASCII table; its line breaks are the content.
      if (/^\s{4,}\S/.test(para)) {
        // Re-indent only. The caller already stripped the block's two-space base off every line,
        // so stripping again here left tables at two spaces relative — under markdown's four, so
        // the columns collapse into a paragraph. Caught by reading the parsed YAML rather than the
        // file, which is where the relative indent is what the renderer will see.
        return para
          .split("\n")
          .map((line) => (line.trim() ? indent + line : ""))
          .join("\n");
      }
      // Markdown that lives at the *start of a line* — bullets, ordered items, blockquotes — is
      // content, not spacing. Rewrapping such a paragraph as prose slides the marker into the
      // middle of a line, where it stops being markup: two bullets render as one item, seven
      // numbered points as one paragraph, a block quotation as prose with stray `>` in it. All of
      // that parses, conserves the word count, and is invisible in the diff.
      //
      // A quotation is one flow with a marker on every line, so it rewraps as a unit and the
      // marker goes back on each line. A list is several flows with a marker on the first line of
      // each, so it splits at the markers and each item wraps under its own.
      if (QUOTE.test(para)) {
        const body = para.replace(/^ *> ?/gm, "").replace(/\n/g, " ");
        return wrap(body, "", "")
          .split("\n")
          .map((line) => `${indent}> ${line}`)
          .join("\n");
      }
      const marker = MARKER.exec(para);
      if (marker) {
        return para
          .split(new RegExp(`\\n(?=${marker[1] === "." ? " *\\d+\\. " : " *[-*] "})`))
          .map((item) => {
            const width = (/^ *(?:[-*]|\d+\.) /.exec(item)?.[0].trimStart().length ?? 2) + 1;
            return wrap(item, indent, indent + " ".repeat(width));
          })
          .join("\n");
      }
      return wrap(para, indent, indent);
    })
    .join("\n\n");
}

/** Greedy wrap to the corpus's 95 columns, first line at `indent`, the rest at `hanging`. */
function wrap(text, indent, hanging) {
  const lines = [];
  let line = indent;
  for (const word of text.split(/\s+/).filter(Boolean)) {
    const at = lines.length === 0 ? indent : hanging;
    if (line !== at && line.length + 1 + word.length > 95) {
      lines.push(line);
      line = hanging + word;
    } else {
      line = line === at ? at + word : `${line} ${word}`;
    }
  }
  lines.push(line);
  return lines.join("\n");
}

/**
 * The block header for some folded content: `|` normally, `|2` when it starts indented.
 *
 * A YAML block scalar takes its indentation from its **first non-empty line**. So a `findings:`
 * whose first paragraph is an ASCII table opens a block indented six, and the next ordinary
 * paragraph at two is *less* indented — which ends the scalar and leaves the rest of the file
 * being parsed as mappings. The node stops loading entirely.
 *
 * The explicit indentation indicator pins it regardless of what the first line looks like. Only
 * emitted where it is needed, so the diff on every other node stays empty.
 */
function header(folded, indent) {
  return /^\s{2}\s+\S/.test(folded.split("\n")[0] ?? "") ? `|${indent.length}` : "|";
}

/** The raw text of a top-level block scalar field, and where it sits in the file. */
function block(raw, field) {
  const open = new RegExp(`^${field}: \\|\\d?\\n`, "m").exec(raw);
  if (!open) return null;
  const start = open.index + open[0].length;
  const rest = raw.slice(start);
  const end = /^\S/m.exec(rest);
  return { start, end: start + (end ? end.index : rest.length) };
}

const plan = JSON.parse(readFileSync(process.argv[2], "utf8"));
let done = 0;

for (const [id, spec] of Object.entries(plan)) {
  const file = join(CORPUS, `${id}.yml`);
  let raw = readFileSync(file, "utf8");
  const at = block(raw, "description");
  if (!at) {
    console.log(`  ! ${id}: no description block`);
    continue;
  }

  const body = raw.slice(at.start, at.end).replace(/\n+$/, "");
  const paras = body.split(/\n\n+/).map((p) => p.replace(/^ {2}/gm, "").trimEnd());

  const toFindings = new Set(spec.findings ?? []);
  const toDrop = new Set(spec.drop ?? []);
  const bad = [...toFindings, ...toDrop].filter((i) => i >= paras.length);
  if (bad.length) {
    console.log(`  ! ${id}: index out of range ${bad} (has ${paras.length})`);
    continue;
  }

  const kept = paras.filter((_, i) => !toFindings.has(i) && !toDrop.has(i));
  const moved = paras.filter((_, i) => toFindings.has(i));
  if (moved.length === 0 && toDrop.size === 0 && !spec.revisions) {
    console.log(`  ! ${id}: nothing to move`);
    continue;
  }

  const keptText = fold(kept.join("\n\n"), "  ");
  // The header may have to change too: a description whose new first paragraph is a table needs
  // the same explicit indicator its `findings:` sibling does.
  const descHeader = new RegExp(`^description: \\|\\d?$`, "m");
  let out =
    raw.slice(0, at.start).replace(descHeader, `description: ${header(keptText, "  ")}`) +
    keptText +
    "\n" +
    raw.slice(at.end);

  if (moved.length) {
    const existing = block(out, "findings");
    if (existing) {
      const current = out.slice(existing.start, existing.end).replace(/\n+$/, "");
      out =
        out.slice(0, existing.start) +
        current +
        "\n\n" +
        fold(moved.join("\n\n"), "  ") +
        "\n" +
        out.slice(existing.end);
    } else {
      // After `description:`, before whatever follows it.
      const desc = block(out, "description");
      const movedText = fold(moved.join("\n\n"), "  ");
      out =
        out.slice(0, desc.end) +
        `findings: ${header(movedText, "  ")}\n` +
        movedText +
        "\n" +
        out.slice(desc.end);
    }
  }

  if (spec.revisions) {
    const entries = spec.revisions
      .map((r) =>
        ["was", "now", "found_by", "reach"]
          .filter((k) => r[k])
          .map((k, i) => `${i === 0 ? "  - " : "    "}${k}: |\n${fold(r[k], "      ")}`)
          .join("\n"),
      )
      .join("\n");

    /*
     * Append to an existing block rather than opening a second one.
     *
     * A node corrected in an earlier pass already has `revisions:`, and inserting unconditionally
     * wrote the key twice — which is a YAML duplicate-key error, so the node stops parsing
     * altogether. Every consumer sees it; none of them can say which entry was meant. Caught by
     * the loader refusing the file, which is the one failure mode of this tool that announces
     * itself.
     */
    const existing = /^revisions:\n/m.exec(out);
    if (existing) {
      const start = existing.index + existing[0].length;
      const rest = out.slice(start);
      const end = start + (/^\S/m.exec(rest)?.index ?? rest.length);
      out = `${out.slice(0, end).replace(/\n+$/, "")}\n${entries}\n${out.slice(end)}`;
    } else {
      const anchor = out.indexOf("\nproperties:");
      out = `${out.slice(0, anchor)}\nrevisions:\n${entries}${out.slice(anchor)}`;
    }
  }

  /*
   * Parse what was written before accepting it, and diff the word counts.
   *
   * Every defect this tool has had corrupted its output rather than merely mis-shaping it: tables
   * dropped below markdown's indent threshold, a `revisions:` key written twice into an
   * unparseable duplicate, a hand edit that spliced a description into a revision entry. Not one
   * was visible in the diff, and all three were found by parsing the result afterwards — so the
   * parse belongs here, before the file is accepted, rather than in whoever remembers to look.
   *
   * The word check is the one that caught the splice: text moving between fields conserves words,
   * so a total that changes by more than the folding can explain means something was duplicated
   * or eaten.
   */
  const before = YAML.parse(raw);
  let after;
  try {
    after = YAML.parse(out);
  } catch (error) {
    // Written aside rather than discarded: a refusal that leaves nothing to look at makes the
    // next step guesswork, and the failure is in the output by definition.
    const wreck = join("/tmp", `${id.replace(/\//g, "-")}.broken.yml`);
    writeFileSync(wreck, out);
    console.log(`  ! ${id}: the result does not parse — ${String(error).split("\n")[0]}`);
    console.log(`      wrote ${wreck} to look at`);
    continue;
  }

  const count = (node) =>
    ["description", "findings"]
      .map((k) => (node[k] ? node[k].split(/\s+/).filter(Boolean).length : 0))
      .reduce((a, b) => a + b, 0);
  const dropped = new Set(spec.drop ?? []);
  const allowed = [...dropped].reduce(
    (sum, i) => sum + paras[i].split(/\s+/).filter(Boolean).length,
    0,
  );
  const lost = count(before) - count(after);
  if (Math.abs(lost - allowed) > 2) {
    console.log(
      `  ! ${id}: ${lost} words left description+findings but ${allowed} were dropped — ` +
        "something was duplicated or eaten, not moved",
    );
    continue;
  }

  // An ASCII table under four spaces renders as a paragraph with its columns collapsed.
  for (const field of ["description", "findings"]) {
    for (const para of (after[field] ?? "").split(/\n\n+/)) {
      if (/^ {2,3}\S/.test(para) && /\s{3,}\S/.test(para.split("\n")[0] ?? "")) {
        console.log(`  ! ${id}: a table in ${field}: sits under four spaces and will collapse`);
      }
    }
  }

  // Line-leading markup lost to rewrapping. A bullet, an ordered item and a quotation marker are
  // all content, and moving text between fields conserves every one of them — but a rewrap that
  // slides one into the middle of a line still parses and still counts the same words, so this is
  // the only check that sees it. Count each kind separately: a bullet becoming a quote would net
  // to zero against a single total, and that is exactly the sort of thing a rewrap does.
  const KINDS = { bullet: /^ *[-*] /gm, ordered: /^ *\d+\. /gm, quote: /^ *> /gm };
  let broke = false;
  for (const [kind, pattern] of Object.entries(KINDS)) {
    const marks = (text) => (String(text ?? "").match(pattern) ?? []).length;
    const total = (n) => marks(n.description) + marks(n.findings);
    // A dropped paragraph takes its markers with it, exactly as it takes its words.
    const shed = [...dropped].reduce((sum, i) => sum + marks(paras[i]), 0);
    const [was, now] = [total(before) - shed, total(after)];
    if (was !== now) {
      console.log(`  ! ${id}: ${was} ${kind} markers became ${now} — a rewrap ran lines together`);
      broke = true;
    }
  }
  if (broke) continue;

  writeFileSync(file, out);
  done += 1;
  console.log(
    `  ${id}: ${kept.length} kept, ${moved.length} to findings` +
      `${toDrop.size ? `, ${toDrop.size} dropped` : ""}` +
      `${spec.revisions ? `, ${spec.revisions.length} revision(s)` : ""}`,
  );
}

console.log(`${done} nodes partitioned`);
