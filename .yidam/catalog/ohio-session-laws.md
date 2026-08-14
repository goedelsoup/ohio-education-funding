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

## What is wired, and the reason the other two are not

H.B. 215 and H.B. 282 are wired. Both print their table once and both reconcile: every parsed row
sums back to the act's own fund-group totals and those to its grand total, in both columns.

H.B. 650 and H.B. 770 are not, and it is a layout problem rather than a retrieval one. Both amend
Section 50 and both print every changed row **twice** — the struck figure inline and the inserted
one on a continuation line beneath it. Worse, **H.B. 650 reprints H.B. 215's fund-group totals
unchanged**, so a reader that reconciles against printed totals passes against the superseded
number. Reading them needs a reader that can tell a struck row from an inserted one, and the
reconciliation that guards every other extraction here would not catch it getting that wrong.

**H.B. 650 also carries a second education table that was never law.** Its SECTION 31 appropriates
$15,000,000 from a School Trust Fund, conditional on the 5 May 1998 ballot measure amending Article
XII Section 14. That measure failed. Its line items are printed as literal placeholders — `XXX
200-YYY` — so nothing here would parse them as amounts, but a text scraper will find the table.

## Two defects in the acts' own arithmetic

- **H.B. 282's printed GRF total for FY2001 is one dollar more than its own fifty-two rows**, while
  its five group totals sum to its printed grand total exactly. The dollar sits between the rows
  and the GRF footing. It is carried as a named defect rather than absorbed by a tolerance.
- **H.B. 650 and H.B. 770 print an FY1999 grand total that omits the Education Improvement Fund's
  $1,443,401**, while their FY1998 grand totals include it. The rows are right and the footing is
  wrong: `appropriation-lines.csv` already carries `200689 Hazardous Waste Removal` as a FY1999
  *actual* of exactly $1,443,401, from the H.B. 94 greenbook. Neither act is wired, so this is
  recorded for whoever wires them.

## Used by

- [`crates/connect/src/fixtures.rs`](../../crates/connect/src/fixtures.rs), `build_session_laws`
- `crates/project/fixtures/session-law-lines.csv`, and `project::session_laws` over it
- The [`fiscal-period`](../corpus/fiscal-period/) nodes for FY1998-FY2001, which had no
  appropriating act named
