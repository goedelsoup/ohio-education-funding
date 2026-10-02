/**
 * The words a chart writes for itself: its axis titles, its tooltips and its name.
 *
 * Each caller used to compose these in whatever form the sentence it was lifted from had, and it
 * showed from chart to chart — an axis titled in lower case on one page and sentence case on the
 * next, a tooltip that was two full axis titles glued together with colons, a name that read
 * "down to 0.." (#612). One place to say how, so a new chart starts from the rule.
 *
 * No imports, on purpose: `chart.ts` uses this and is in every page's bundle.
 */

/**
 * Words a chart prints on its own, in sentence case.
 *
 * Applied where `plot/spec.ts` draws every axis title, foot and end label rather than at the call
 * sites, so a new chart cannot reintroduce the split. Only a lower-case first letter moves:
 * "FY2024", "$1,300" and "ADM" are already what they should be.
 */
export function sentence(text: string): string {
  const first = text.charAt(0);
  return first === first.toLowerCase() && first !== first.toUpperCase()
    ? first.toUpperCase() + text.slice(1)
    : text;
}

/**
 * A sentence with its closing full stop taken off, for joining to another.
 *
 * A name ending in a period and then joined with ". " read "down to 0.." on `/bounds`, once the
 * cursor's instruction was added to it.
 */
export function unstop(text: string): string {
  return text.trimEnd().replace(/\.+$/, "");
}

/**
 * What a tooltip calls an axis: the title up to its first colon or comma, in the middle of a
 * sentence.
 *
 * Axis titles are written to stand under an axis on their own — "The enrollment term: fewer pupils
 * than in FY2020, in logs" — and a hover that joined two of them ran past two hundred characters
 * with two colons in it. The qualifiers after the head are the axis's to say, and it does. A
 * leading article goes, and the first letter is lowered unless its word is an initialism: lowering
 * the whole title wrote "enrolled adm".
 */
export function axisShort(label: string): string {
  const head = (label.split(/[:,]/)[0] ?? label).trim().replace(/^the\s+/i, "");
  const word = head.split(/\s/)[0] ?? "";
  return word.slice(1) === word.slice(1).toLowerCase()
    ? head.charAt(0).toLowerCase() + head.slice(1)
    : head;
}

/**
 * A note on a reading, after it rather than instead of it.
 *
 * A manifest row's `hover` replaced the label and value outright, so the EdChoice "2024-25" bar's
 * tooltip was only "The bottom-fifth window is cut from two rankings here, not three" and never
 * said what the bar is. A note qualifies a figure, so the figure comes first.
 */
export function noted(reading: string, note: string | undefined): string {
  return note ? `${reading} — ${note}` : reading;
}

/**
 * A point's hover, in the one pattern every composed cloud, plane and spread uses:
 * `Name — x: value; y: value`, and the point's class or status after another dash.
 */
export function pointHover(
  name: string,
  x: { label: string; value: string },
  y: { label: string; value: string },
  note?: string,
): string {
  return noted(
    `${name} — ${axisShort(x.label)}: ${x.value}; ${axisShort(y.label)}: ${y.value}`,
    note,
  );
}

/**
 * Words a chart may not print, each with what to write instead.
 *
 * The quantity paid had three names on three charts — "Realized aid per pupil", "as received",
 * "What the district receives" — and two of the site's charts printed a source column's initialism
 * the page defined only after the chart or not at all: "On CE" on /history, "Enrolled ADM, the
 * single year" on /bounds (#658). The terms stay in prose and method notes, where a sentence can
 * say what they are; a line end or a gutter cannot.
 */
export const BANNED: { word: RegExp; instead: string }[] = [
  { word: /\brealiz/i, instead: `"aid received", the site's one name for aid actually paid` },
  { word: /\bCE\b/, instead: `"access", and name the count in a note` },
  { word: /\bADM\b/, instead: `"enrollment" or "pupils", and expand ADM in a note` },
];

/** What a chart's text says that it should not, one entry per banned word it contains. */
export function banned(text: string): string[] {
  return BANNED.filter((b) => b.word.test(text)).map(
    (b) => `${text.match(b.word)![0]} in "${text}": write ${b.instead}`,
  );
}
