# Scholarship Annual Report — the department's own account of the channel

**Source.** Ohio Department of Education and Workforce, *2025 Scholarship Annual Report*, covering
the 2024-2025 school year.
**Type.** Primary source, published by the programme administrator.
**Location.** `https://education.ohio.gov/getattachment/About/Annual-Reports/2025-Scholarship-Annual-Report.pdf.aspx?lang=en-US`.
About 1.3 MB, 21 pages, retrievable by a self-identifying agent.

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
