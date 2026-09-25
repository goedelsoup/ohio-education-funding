# Scholarship Annual Report — the department's own account of the channel

**Source.** Ohio Department of Education and Workforce, *2025 Scholarship Annual Report*, covering
the 2024-2025 school year.
**Type.** Primary source, published by the programme administrator.
**Location.** `https://education.ohio.gov/getattachment/About/Annual-Reports/2025-Scholarship-Annual-Report.pdf.aspx?lang=en-US`.
About 1.3 MB, 22 pages, retrievable by a self-identifying agent.

**What it contains.** One section per programme — Traditional EdChoice, EdChoice Expansion,
Cleveland, Autism, Jon Peterson Special Needs — each with a summary, the authorising Revised Code
sections, eligibility in prose, the fiscal-year maximum award, participation, statewide
expenditure, a participation-by-year chart, and a provider list.

The figures this corpus takes from it, all for 2024-2025:

| Programme | Students | Expenditure | Average award, as published |
|---|--:|--:|--:|
| Traditional EdChoice | 42,607 | $283,149,735.14 | $6,808.75 |
| EdChoice Expansion | 100,939 | $492,867,235.98 | $4,958.41 |
| Cleveland | 8,345 | $55,155,295.70 | $6,836.35 |
| Autism | 6,016 | $160,018,883.30 | $26,598.88 |
| Jon Peterson | 8,680 | $103,944,388.15 | not published |

Jon Peterson's expenditure is **derived**, not quoted: it is the sum of the report's six
disability-category figures, which the report displays as a chart without totalling. The other four
are quoted.

**Why it matters here.** It is the only committed source that sizes the scholarship channel from
the department rather than from statute. The four programmes that publish a total come to
**$991,191,150.12**; adding Jon Peterson's derived $103,944,388.15 brings the channel to
**$1.095 billion**, across **166,587 students**. The two figures are kept apart because the round
one is the quotable one and only the first is fully sourced — which is still the order of magnitude
the
[`deduction`](../skills/deduction.md) stub reasoned about when it ruled out a deduction hiding
inside a $95.6 million residual.

## Two things in it that do not reconcile

Recorded because both are visible in the document's own numbers, and neither is explained in it.

**The executive summary overstates its own parts.** It says the five programmes "collectively
empowered the families of more than 175,000 students". The five programme counts it publishes sum
to **166,587** — a gap of 8,413, about 5%. A student holding scholarships in two programmes would
push the sum the other way, so double-counting does not explain it.

**Three of the four published averages are not expenditure over participation.**

| Programme | Published average | Expenditure ÷ participants |
|---|--:|--:|
| Traditional EdChoice | $6,808.75 | $6,645.62 |
| EdChoice Expansion | $4,958.41 | $4,882.82 |
| Cleveland | $6,836.35 | $6,609.38 |
| Autism | $26,598.88 | **$26,598.88** |

Autism matches to the cent and the other three do not, each published figure sitting above the
implied one by 1.5–3.4%. That pattern says denominator rather than arithmetic error — plausibly an
average over students holding a full-year award rather than over everyone who used the programme
at any point during the year. **The report does not say**, so this corpus records the discrepancy
and does not adopt the explanation.

## There is one edition, and the archive says so

Asked whether a 2024 or earlier edition exists at a renamed address, this record now answers from
the listing rather than from a filename pattern.

The [Annual Reports](https://education.ohio.gov/About/Annual-Reports) page was read in full on
24 September 2026: **64 `getattachment` links, exactly one of them a Scholarship Annual Report**.
The Wayback Machine holds that page **74 times between 19 July 2017 and 17 June 2026**, and its
index of everything ever captured under `/getattachment/About/Annual-Reports/` runs to **238
distinct files — of which exactly one carries "Scholarship" in its name**, this one. So the
absence is not a naming change and not a page this project failed to scrape: no consolidated
scholarship report of any earlier vintage has ever been served from this directory.

**What precedes it is a per-programme lineage, not an earlier edition of the same thing.** The
archive's index holds `FY2016-JPSN-Board-Report.pdf`, `FY2017-JPSN-Annual-Report.pdf`,
`JPSN-Report-FY20.pdf`, `FY23_JPSN_Annual_Report.pdf` and `FY24_JPSN_Annual_Report.pdf` — the Jon
Peterson programme reporting on itself, as its own authorising section requires — plus
`Ohio-ACE-Educational-Savings-Account-Report.pdf` for a programme this report does not cover at
all. The last two JPSN editions are still on the live page beside this one, and are now extracted:
[JPSN Annual Report](dew-jpsn-annual-report.md) carries FY2023 and FY2024 for that one programme.

That changes what the gap above the FY1997-FY2013 [historical
archive](dew-scholarship-historical-data.md) is. It is not a series this project has not looked
for; there is no consolidated series to find. The consolidated five-programme account begins with
2024-2025, and one programme's own reporting is what covers part of the decade before it — which
is now partly read rather than only named, so the gap in *this* channel is FY2014 through FY2022
and Jon Peterson's is FY2014 through FY2022 as well.

The two editions that are read also correct the shape of that lineage. They were not a rolling
one-at-a-time replacement: three older JPSN editions were served from this page as late as June
2023 and gone by June 2024, and FY23 and FY24 were both exported from Word on one afternoon in
March 2025. A lapse and a catch-up, which the [JPSN record](dew-jpsn-annual-report.md) dates.

## What it is not

**It is not a per-district file, and it points at one this project has not found.** The Jon
Peterson section states that students came from 494 districts, over 80% of Ohio's, and says a
breakdown per district "is available here" — linking to a route on `reports.education.ohio.gov`
that returns 404. This record read that 404 as a withdrawal, and the reading is void: probed
2026-09-15 on [#10](https://github.com/goedelsoup/ohio-education-funding/issues/10), every path on
that host returns the byte-identical Angular shell, `/robots.txt` included, and the reports behind
it are Power BI embeds behind an entitlement the portal's anonymous token does not carry. A status
code there is a property of the portal, not evidence about the file. What stands is narrower: the
breakdown is cited by a current departmental document, and nothing this project can fetch carries
it. The [`deduction`](../skills/deduction.md) stub records that route and one other, and what would
reach them.

The second of the two has a public substitute that is not the same file: the
[EdChoice designated list](dew-edchoice-designated-list.md) is per district and per building, and
is eligibility rather than participation. Nothing published answers which district a scholarship
was charged against.

**And it is not the only participation source any more.** The
[historical archive](dew-scholarship-historical-data.md) carries FY1997 through FY2013 statewide,
and counts applications and scholarships-paid separately — which is a second denominator for the
same quantity, and the shape of evidence the guess above needs. It does not identify this report's
denominator; it establishes that the department has long kept two.

**It is not the deduct era.** Every figure is 2024-2025, under the Fair School Funding Plan, where
each scholarship is its own funding unit paid directly. Nothing here bears on what a district lost
when the mechanism was a deduction; that needs `dew-payment-reports`.
