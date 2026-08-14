# Ohio session laws — the appropriation acts themselves

**Source.** Ohio General Assembly, via the Legislative Information Systems service that backs
`legislature.ohio.gov`.
**Type.** Primary source — the enrolled act, which is the text that was voted on and signed.
**Location.** `search-prod.lis.state.oh.us/api/v2/general_assembly_{GA}/legislation/{bill}/{version}/pdf/`.

**What it contains.** The whole act. For a main operating budget that is on the order of fifteen
hundred pages, and the part this repository reads is one agency's appropriation table — for
Am. Sub. H.B. 215 of the 122nd, `SECTION 50. EDU DEPARTMENT OF EDUCATION`, four pages of it.

    GRF 200-501 School Foundation Basic         $     2,202,851,688 $              0
                Allowance
    GRF 200-901 Property Tax Allocation -       $      566,800,000 $     600,800,000

A fund code, the hyphenated line item, a title that wraps, and one money column per fiscal year of
the biennium. The modern series writes the same item `200501`.

**Why this and not an analysis of it.** Everything else in the appropriation series is the
Legislative Service Commission describing what an act did — greenbooks, budget workbooks, the
Catalog of Budget Line Items — and all three stop at FY2002 or later. The acts stop where the
legislature's own archive stops, and that is four fiscal years earlier.

## The floor is the publisher's, and it is stated rather than inferred

`GET https://search-prod.lis.state.oh.us/api/v2/` returns the service's own index of every General
Assembly it holds. **It has seventeen entries and the oldest is the 122nd**, 1997-01-06 to
1998-12-31. There is nothing below it, in any version code or naming.

So the pre-FY2002 record reaches **FY1998 and no further**. *DeRolph I* was decided in March 1997
and falls inside the 122nd, so the first budget enacted after it is reachable. The Foundation
Program era and the equal yield formula are not, and not because nobody has looked: the body that
publishes them does not publish them.

## The version code is not guessable

It is the bill's position in its own version sequence, so it differs per bill: H.B. 215 is `06_EN`,
H.B. 650 and H.B. 770 are `05_EN`, and H.B. 282 is `08_EN` because two interim postings sit ahead
of it. Guessing `06_EN` returns a hard 404 with a nine-byte body, which reads exactly like "not
served". The index is `.../legislation/{bill}/`, a JSON array with one object per version carrying
`formatted_version` and `version_id`; the PDF path is those two joined by an underscore.

## Four acts carry a Department of Education table

| act | biennium | what it is |
|---|---|---|
| Am. Sub. H.B. 215, 122nd | FY1998-99 | the main operating budget |
| Am. Sub. H.B. 650, 122nd | FY1999 | itemises the year H.B. 215 left in one line |
| Am. Sub. H.B. 770, 122nd | FY1998-99 | corrective; reprints Section 50 as amended |
| Am. Sub. H.B. 282, 123rd | FY2000-01 | education appropriations, and **not** the budget bill |

**The 123rd took education out of the operating budget.** Am. Sub. H.B. 283 is that biennium's
budget, 977 pages, and contains no Department of Education section and no `200-5xx` line at all.
Education was appropriated by H.B. 282, enacted a day earlier, as the response to *DeRolph II*. So
"which act carries the education table" is not answered by "the operating budget" for every year.

## What is wired, and how an amending act is read

**H.B. 215, H.B. 770 and H.B. 282.** All three reconcile: every parsed row sums back to the act's
own fund-group totals and those to its grand total, in both columns.

**H.B. 770 is the operative text for FY1998-99 and H.B. 215 is not.** It reprints Section 50 *as
already amended by H.B. 650*, so its columns carry the itemisation H.B. 215 deferred, with
`200-405` struck back to zero. H.B. 215 is kept because the shape of what it did is the finding.

The amendment is **positional, not typographic**. Strike-through does not survive into the text
layer and does not need to: the row line carries the figures as they stood, and H.B. 770's own
replacements are printed beneath, right-aligned under the column each replaces.

    GRF 200-100   Personal Services   $  10,744,925 $  10,756,210
                                                       11,256,210

So a replacement is a line of nothing but digits whose numbers end where the row's amounts end.
That last condition is what keeps page furniture out — the acts print a bare page number on its own
line, and `247` under a row whose columns end at 59 and 74 amends nothing.

**A replacement can sit two lines down**, after the wrapped half of its own title. `200-545
Vocational Education Enhancements` is amended from $184,298,314 to $201,991,432 that way, and a
reader that only looks at the next line loses $17,693,118 — which is how this one was found.

**H.B. 650 is deliberately not wired.** Its own table is the strike-and-insert edit that H.B. 770
reprints in settled form, and it reprints H.B. 215's fund-group totals **unchanged** — so a reader
reconciling against printed totals would pass against the superseded number. Everything it
establishes is legible in H.B. 770 without that hazard.

**H.B. 650 also carries a second education table that was never law.** Its SECTION 31 appropriates
$15,000,000 from a School Trust Fund, conditional on the 5 May 1998 ballot measure amending Article
XII Section 14. That measure failed. Its line items are printed as literal placeholders — `XXX
200-YYY` — so nothing here would parse them as amounts, but a text scraper will find the table.

## Two defects in the acts' own arithmetic

- **H.B. 282's printed GRF total for FY2001 is one dollar more than its own fifty-two rows**, while
  its five group totals sum to its printed grand total exactly. The dollar sits between the rows
  and the GRF footing. It is carried as a named defect rather than absorbed by a tolerance.
- **H.B. 770's FY1999 grand total omits the Education Improvement Fund's $1,443,401**, while its
  FY1998 grand total includes it. This one is resolvable in the rows' favour and was resolved
  there: `appropriation-lines.csv` already carries `200689 Hazardous Waste Removal` as a FY1999
  *actual* of exactly $1,443,401, from the H.B. 94 greenbook, so the money was appropriated and
  spent and it is the act's footing that is wrong.

## One line renumbered twice in four years

The lottery half of the formula is `017 200670 School Foundation - Basic Allowance` in FY1998,
`017 200610 Base Cost Funding` in FY1999, and `017 200612` from FY2000 — the same money in the same
fund under three numbers. The FY1999 identity is confirmed from outside the acts: the greenbook's
FY1999 *actual* for `200612` is $666,093,028, which is what H.B. 770 appropriates `017 200610`, to
the dollar.

And `200610` is not a key on its own: `454 200610` is `Guidance and Testing` in the same tables,
half a million a year. Anything reading the formula's share here must key on fund and number
together.

## Used by

- [`crates/connect/src/fixtures.rs`](../../crates/connect/src/fixtures.rs), `build_session_laws`
- `crates/project/fixtures/session-law-lines.csv`, and `project::session_laws` over it
- The [`fiscal-period`](../corpus/fiscal-period/) nodes for FY1998-FY2001, which had no
  appropriating act named
