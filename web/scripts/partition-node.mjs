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

const CORPUS = "/Users/cparent/Code/goedelsoup/ohio-education-funding/main/.yidam/corpus";

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
      const lines = [];
      let line = indent;
      for (const word of para.split(/\s+/).filter(Boolean)) {
        if (line !== indent && line.length + 1 + word.length > 95) {
          lines.push(line);
          line = indent + word;
        } else {
          line = line === indent ? indent + word : `${line} ${word}`;
        }
      }
      lines.push(line);
      return lines.join("\n");
    })
    .join("\n\n");
}

/** The raw text of a top-level block scalar field, and where it sits in the file. */
function block(raw, field) {
  const open = new RegExp(`^${field}: \\|\\n`, "m").exec(raw);
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

  let out = raw.slice(0, at.start) + fold(kept.join("\n\n"), "  ") + "\n" + raw.slice(at.end);

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
      out =
        out.slice(0, desc.end) +
        "findings: |\n" +
        fold(moved.join("\n\n"), "  ") +
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

  writeFileSync(file, out);
  done += 1;
  console.log(
    `  ${id}: ${kept.length} kept, ${moved.length} to findings` +
      `${toDrop.size ? `, ${toDrop.size} dropped` : ""}` +
      `${spec.revisions ? `, ${spec.revisions.length} revision(s)` : ""}`,
  );
}

console.log(`${done} nodes partitioned`);
