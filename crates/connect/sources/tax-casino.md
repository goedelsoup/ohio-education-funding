# tax-casino — the gross casino revenue county student fund

**What it is for.** To size the one education channel in this corpus that reaches every district
and appears in no appropriation table.

`casino-tax-distribution` carried the same open item through four phases: whether the General
Assembly set foundation aid lower because casino money reaches districts outside the formula. The
appropriation history was retrieved and could not answer it — in constant dollars foundation aid
moves by a median of $236 million a year, and a substitution smaller than that is arithmetically
inside the noise. The node then said what would settle it: **the casino distribution's own size**,
to establish whether the channel is even of a magnitude the total could register.

This connector is that. It does not make the substitution question answerable; it makes the
*unanswerability* a measurement. At $114.2 million in its largest fiscal year the whole channel is
under half the noise floor, so no appropriation series will ever show it, however long the series
gets.

**Why it was declared blocked, and why it was not.** The issue that recorded the blocker said both
publishers 404 at their document root to a self-identifying agent, which is true and is not the
same as the documents being unreachable. `tax.ohio.gov` serves its pages to a browser, and it links
every file under `/static/`, which resolves to `dam.assets.ohio.gov/raw/upload/tax.ohio.gov/…` —
the route [`tax-abstract`](tax-abstract.md) has used since it was wired, documented in
`dot-sd1-school-district-taxes.md`, and never applied here. The Internet Archive was named as the
cheap untried option; it was not needed. Every one of these files is live, and the page that lists
them is live too, which is what makes the end of the series readable rather than inferred.

**What it retrieves.** Sixteen workbooks holding eighteen half-yearly distributions, August 2015
through January 2024 — nine complete state fiscal years. Fifteen are the county-by-district layout;
the sixteenth is a combined workbook whose four sheets are the only published form of the August
2015, January 2016, August 2016 and January 2017 distributions.

**What it cannot reach, and why that is an answer.** Nothing after January 2024, because the
department's casino page carries nothing after January 2024 in any category. Nothing before August
2015 in a machine-readable form: those distributions are `Final SD Distribution` PDFs. The gap at
the recent end is a publisher's silence rather than a retrieval problem, and the connector's
`still_blocked` says so — a refresh will not find more until the department posts more.

**What checks it.** The department prints its own total at the foot of every sheet, so each of the
eighteen is reconciled against itself before the fixture is written; fifteen also print the
half-year they cover, which is checked against the payment month; and August 2015 is published
twice in two layouts, which are checked against each other across 1,044 districts. See
[`build_casino_extract`](../src/fixtures.rs). That property — an aggregate stated beside its parts
— is the one worth looking for in any new source, and it is what a plausible figure in a rising
series cannot substitute for.
