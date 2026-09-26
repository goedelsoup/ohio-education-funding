# Foundation Funding Bases — the two bases the phase-in runs from

**Source.** Ohio Department of Education and Workforce, Office of Budget and School Funding.
One workbook, `Foundation-Funding-Bases-6-16-2022.xlsx`.
**Type.** Primary source — the department's own published bases, with the methodology it used to
build them stated on the face of the file.
**Location.** `education.ohio.gov`, Finance and Funding → School Payment Reports → State Funding
for Schools → Traditional School Districts. Openly served, no credentials, 499,238 bytes.
**The current department page no longer links it.** The file is still served at its address. A
digest is pinned in [`source-digests.txt`](../../crates/connect/source-digests.txt) because an
unlinked file is one nobody will announce a revision of.

**What makes it load-bearing is the `Introduction` sheet, not the numbers.** The Fair School Funding
Plan phases in against two bases — `[H2]`, the FY2020 base state funding the temporary transitional
aid guarantee `[I]` compares against, and `[L1]`, the FY2021 funding base `[K] Formula Transition
Supplement` holds a district at. Statute names both and defines neither arithmetically. This sheet
does, line by line, in the department's words:

- `[H2]` is the FY2020 May #1 formula payment (line A), **less** FY2019 final capped transportation,
  **less** the FY2020 community and STEM school deduction, **less** the FY2020 total scholarship
  transfer, **plus** FY2020 net open enrolment for K-12 students, **plus** the FY2021 net excess cost
  adjustment — that last term removed for FY2023 by H.B. 583.
- `[L1]` is the FY2021 final #2 formula payment (line A) with the executive budget reductions
  restored, **plus** FY2021 net open enrolment, **plus** FY2022 net excess cost, **less** the FY2021
  community and STEM school deduction, **less** the FY2021 total scholarship transfer, **plus** FY2021
  student wellness and success, **plus** the FY2021 enrolment growth supplement.

Three sentences on that sheet answer questions the acts leave silent.

**Why FY2019 figures stand in for FY2020 ones:** *"References to FY19 funding are used because FY20
state foundation funding for traditional districts was flat to FY19 funding for the categories
described below."* That is the licence for reading a 2019 payment report as the base of a 2022
formula.

**Why `[L1]` is larger than `[H2]` by a district's transportation:** *"transportation funding is not
subject to the phase-in and is therefore removed from the base state funding calculation."* The gap
is a subtraction one definition makes and the other does not — the phase-in's own scope, not an
accident of which components were capped.

**Where the guarantee went:** *"any eliminated funding elements from the previous formula (such as
K-3 literacy) or elements not specifically mentioned (such as the guarantee or EdChoice scholarship
deductions) default to the funding base used for line A Base Cost."* The department says in its own
voice that the retired guarantee is **inside** the base and unallocated. That is the fact
[`scenario/guarantee-phase-out`](../corpus/scenario/guarantee-phase-out.yml) needed to price the
third reading of H.B. 110 Section 265.225, and no statute states it.

**What it contains.** Six worksheets and one named range: `Introduction`, `Core Foundation Funding
(Ha)`, `Components (Aa-Ga) - Jan FY22`, `Components (Aa-Ga) - Feb FY22`, `Components (Aa-Ha) FY23`,
and `FY21 Funding Base`. The last is 11 columns by 618 rows — the seven terms of `[L1]`, their sum,
and IRN, district and county — carrying 611 districts in rows 7 through 617, sorted by district
name. `docProps` credits Shams, Daria, dates creation to 2022-01-12 and last modification to
2022-06-16, and names Sanders, Elena as the last editor; the filename's own date is the
modification. No sheet is protected.

The seven terms of `[L1]` sum to `FY21 FUNDING BASE` to the cent on all 611 rows, and the first term
equals the same district's `TOTAL FOUNDATION FUNDING` in the FY2019 final payment report on 609 of
them — see [`dew-fy19-payment-report`](dew-fy19-payment-report.md) for the two exceptions and the
one dissolution. A 611-row extract is committed at
[`crates/project/fixtures/fy21-funding-base.csv`](../../crates/project/fixtures/fy21-funding-base.csv),
built by
[`connect::fixtures::transition_base`](../../crates/connect/src/fixtures/transition_base.rs), which
refuses a file that stops reproducing either identity.

## Caveats

**The file holds three vintages of the same base, and they are not interchangeable.** `Components
(Aa-Ga) - Jan FY22` was superseded in February, because the January allocation produced negative
component bases wherever FY2020's negative adjustments exceeded FY2019's amounts; the department set
those to zero and moved the difference into line A Base Cost. `Components (Aa-Ha) FY23` differs
again, because H.B. 583 struck the excess-cost adjustments from the FY2020 base for FY2023 only.
Reading the wrong sheet for a year is not a rounding difference.

**`[L1]`'s `FY22 NET EXCESS COST` term is zero on every row, and that is the file's own admission,
not a nil.** The `Introduction` sheet says *"The FY22 excess cost is not yet available"* — the
workbook was last touched in June 2022. The term exists in the definition and carries no value here.

**The component allocation is the department's, not the law's.** *"While the law directs the total
(line H) funding base, the Department allocated this total amount to each line."* Line H is
statutory; lines A through G are a methodology, and the department published its reasoning because
the funding streams it split the total into are separately restricted.

**One district on the base sheet has no name.** IRN `048959` — Middle Bass Local, an Ottawa County
island district — appears as `#N/A` in both the name and county columns, with zeros across all
seven terms. The row is real and sums correctly; the lookup that failed was the department's. The
extractor takes that district's name from the FY2019 report rather than committing a spreadsheet
error as a district name, and does so only for a name that is literally `#N/A`.

**The total row carries no IRN, and one of its cells is empty.** Row 618 is a statewide sum that the
extractor drops along with every other non-district row, and its community and STEM school deduction
cell is blank where the other seven totals are filled. A reader taking the published statewide total
gets a figure whose terms do not sum to it.

**The `FY21` on the community and STEM deduction column sits alone, two rows above the rest of its
label.** Any header rule that treats a single-cell row as a banner silently strips the year off that
one column of the seven. See the header discussion in
[`connect::fixtures::transition_base`](../../crates/connect/src/fixtures/transition_base.rs).
