/**
 * One collation for the whole site, so a build is the same build wherever it runs.
 *
 * # What was wrong
 *
 * `localeCompare` was called with no locale argument at sixteen sites, including the sort behind
 * the 609-row district index and the CSV `/data` publishes. With no locale the runtime uses the
 * one it inherited from the environment — `LANG`, `LC_ALL` — so what the site *says* depended on
 * the machine that built it.
 *
 * Measured rather than assumed, over the 1,133 district names, county names, corpus labels, titles
 * and slugs this site actually sorts: **nine of thirty locales order them differently.** Czech and
 * Slovak treat `ch` as a single letter that follows `h`, so a `cs_CZ` build moves *Chagrin Falls*,
 * *Champion*, *Chardon*, *Chesapeake Union*, *Chillicothe* and *Chippewa* about a hundred and
 * forty rows down the published CSV. Latvian and Lithuanian sort `y` with `i`, which moves *Akron
 * City* past *Ayersville Local*. Estonian puts `z` between `s` and `t`. Danish and Turkish order
 * the cases differently. Building the site under `cs_CZ.UTF-8` against `en_US.UTF-8` produces ten
 * different files, `data/districts.csv` and `districts.html` among them.
 *
 * # Why `en` and not the reader's own language
 *
 * Because the order has to be one order. These are the names of Ohio school districts, written in
 * English, in a table whose rows are also *rendered* in a fixed order at build time — so a client
 * sort in the reader's locale would disagree with the server's the moment they touched a column
 * heading. `districts.ts` is the one of these sixteen that runs in a browser, and it was reading
 * the reader's locale rather than the document's.
 *
 * # Why a collator and not `localeCompare(other, "en")`
 *
 * The sixteen sites can be fixed either way and the orderings agree. A collator is constructed
 * once instead of per comparison, which for a 609-element sort is the difference between one
 * collation object and several thousand; and having one exported name means the next sort is
 * written against something, rather than being the seventeenth site nobody notices.
 *
 * Default options, deliberately. `numeric: true` would be a *change* — it would put `District 9`
 * before `District 10` where today it follows — and this module exists to remove a difference
 * between machines, not to introduce one between builds. An `en` build before and after this is
 * byte-identical; every other locale's build now matches it.
 */

/**
 * Compare two strings the way this site orders text.
 *
 * Use it anywhere a sort is on something a reader reads. `localeCompare` with no locale is
 * forbidden across `web/src/` and `order.spec.ts` holds that at zero.
 */
export const compare: (a: string, b: string) => number = new Intl.Collator("en").compare;
