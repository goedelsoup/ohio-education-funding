# LSC Redbook — Ohio Department of Education and Workforce

**Source.** Ohio Legislative Service Commission, Legislative Budget Office. *Redbook: Ohio
Department of Education and Workforce*, H.B. 96 of the 136th General Assembly, as introduced.
**Type.** Primary source for legislative practice — the legislature's own budget office explaining
the department's appropriations to the members who vote on them.
**Location.** `lsc.ohio.gov/assets/legislation/136/hb96/in/files/`,
`hb96-edu-redbook-as-introduced-136th-general-assembly.pdf`.

**What it contains.** Every appropriation line item in the department's budget, each broken into
its earmarks with amounts, and prose describing what each one funds. It is the only document this
corpus holds that connects a **program** to the **budget line** that pays for it.

**As introduced, and the distinction is load-bearing.** LSC publishes redbooks only for the
introduced bill. The **line item numbers** here are the enacted ones — an ALI is not renumbered in
conference — and the **amounts** are the executive proposal. Every figure quoted from here says so.
The preceding phase nearly published a House-passed supplement schedule as though it were law, and
the habit that catches it is naming the version at the point of quotation rather than in a footnote.

**What it settled.** The [appropriation proration
factor](../corpus/parameter/appropriation-proration-factor.yml) node recorded "when each factor was
last set, and against which appropriation" as needing "the budget bill's line items, which is
`lsc-budget` — a connector still blocked on PDFs." Two findings came out of one table:

- **Preschool special education is the remainder of ALI 200540.** Five earmarks are funded first
  and it takes what is left. A residual claimant absorbs a shortfall by construction, which is
  *why* this program prorates when most do not — and nothing on the department's own sheet says so.
- **The $147,500,000 limit the FY2027 calculator prints is the FY2025 estimate.** FY2026 and FY2027
  are both $153,976,832. So the proration factor and the limit beside it were carried over together
  from the prior biennium, and against the year being modelled the program is $5.6m *under* its
  appropriation rather than $908,184 over it.

It also corroborates, in prose, a claim this corpus first reached by noticing an absence:
"Transportation funds are mostly allocated based on the prior year's costs." R.C. 3317.0212 contains
no dollar figure, which is how the rates were identified as **measured** rather than chosen; the
redbook says the same thing directly.

**What it is not.** Not the enacted appropriations — for those, read the **greenbook** at
[`lsc-hb96-analysis`](lsc-hb96-analysis.md), committed as
[`dew-greenbook.txt`](../../crates/project/fixtures/dew-greenbook.txt). It is the same document as
enacted: same structure, same five-line foundation aid table, columns headed `Appropriation` where
these are headed `Introduced`. $153,976,832 did survive conference, and so did every earmark beside
it — but the foundation aid amounts did not, and Fund 7017 alone came out $97,638,202 above the
proposal. **Quote an amount from here only as a proposal, and only when the greenbook cannot answer
the question.** Not a series either — this is one biennium, and the continuous appropriation-line
record `lsc-budget` exists for is still ahead of it.

**How it is read.** `connect::cache::pdf_text`, which shells out to `pdftotext`. That is a weaker
dependency than the `curl` the rest of retrieval uses — poppler does not ship with macOS or
Windows — so a rebuild without it reports the fixture skipped rather than failing. The extract is
committed, so only a refresh needs poppler.

## Used by

- [`crates/project/fixtures/dew-redbook.txt`](../../crates/project/fixtures/dew-redbook.txt)
- [`parameter/appropriation-proration-factor`](../corpus/parameter/appropriation-proration-factor.yml)

## Feeds connector

[`lsc-budget`](../../crates/connect/sources/lsc-budget.md)
