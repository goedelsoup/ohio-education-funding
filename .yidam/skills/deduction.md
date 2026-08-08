---
name: deduction
description: Compute community school and scholarship diversions against a resident district by program and fiscal year
---

# Calculator: deduction (stub)

**Computes.** The amount and student count leaving a resident district through each
[`program`](../corpus/program/) — EdChoice, Cleveland Scholarship, Jon Peterson, Autism — and
through community and STEM school enrollment.

**Reads.** Program nodes; per-agency scholarship and community school participation series from
`crates/`.

**Returns.** Per resident district, per program, per fiscal year: student count, dollar amount,
and the mechanism in force — deduction or direct appropriation.

## The mechanism switch

Getting this wrong is the most common error in Ohio funding analysis. Before the
[Fair School Funding Plan](../corpus/funding-regime/fair-school-funding-plan.yml), community
school and scholarship funding was deducted from the resident district's foundation payment.
After it, community and STEM students are funded directly by the state. A series that spans the
transition without marking it will show districts' foundation payments rising in FY2022 for
reasons that have nothing to do with the formula.

The calculator must therefore return the mechanism alongside the amount, and must never sum
across the boundary without a note.

The sharpest case in the corpus is
[ECOT](../corpus/education-agency/electronic-classroom-of-tomorrow.yml): deduct-funded
throughout its operation, then subject to recovery for students it could not document. Whether
recovered money returned to the deducted districts is an `[open]` question this calculator
should be able to settle.

## Status

**Stub — not implemented, and now known not to be implementable from this source.** The recorded
blocker was `dew-foundation`, and that was wrong. That connector is now wired with three sources
and the block did not lift, because **the FY2027 funding calculator does not carry the deduction
channel at all**.

That was an inference from the columns the corpus had extracted. It has since been checked
against every column the workbook has, and it holds in the strongest form available:

- **`Summary_SFPR`, thirty columns.** The transfer channel is exactly two lines, `S - Educational
  Service Center` and `T - Other Adjustments`, summing to `U - Total Transfers`. Neither is named
  for a scholarship or a community school.
- **`Detail_SFPR`, fifty-eight columns.** The full formula decomposition, `[a] Enrolled ADM`
  through `[N] Total Formula Funding`. No deduction line.
- **The District Profile Report, sixty-one columns.** None either.

And the one place a deduction could still have been sitting is now measured rather than argued
about. `T - Other Adjustments` is the report's only unlabelled line; its negative half totals
**$95.6 million across 577 districts, 1.12% of total state support.** Ohio's scholarship programs
run on the order of a billion dollars a year. A deduction channel would have to be roughly ten
times the size of the entire residual it would have to hide in.

The `[open]` question "could a deduction be inside the negative transfers" is therefore
**closed**, and closed for the reason the mechanism note above predicts: under the Fair School
Funding Plan community and STEM students are funded directly, so there is no deduction to carry.
`crates/project/tests/the_voucher_channel_is_absent.rs` holds the bound.

**What remains open is the pre-FSFP era**, which is a different question and a different source.
Deduct-era amounts per resident district — and whether ECOT recovery money returned to the
districts it was deducted from — need the department's historical payment reports, which are not
the FY2027 calculator and are not held here. That is a retrieval problem.

### What was searched, and what was found

Recorded so a later phase does not repeat it.

**The FY2027 calculator carries no deduction.** Its only candidate is `U - Total Transfers`,
which is `S - Educational Service Center` plus an unlabelled `T - Other Adjustments`. Direction
rules the line out: a deduction can only reduce, and **20 of 609 districts receive a positive
transfer**. Cleveland Municipal — the district with the most community school enrollment in the
state — has transfers of −$3.8M against $322.6M of total state support, about 1%. Pinned in
[`crates/project/tests/the_voucher_channel_is_absent.rs`](../../crates/project/tests/the_voucher_channel_is_absent.rs).

That is a narrower finding than it sounds. Ruling out the line does not rule out a deduction
inside its negative half — Shawnee Local's transfers are 44% of its total state support, which is
a great deal of service centre — and the report does not label the components. **[open]**

**`public.education.ohio.gov` has no deduction directory.** The host serves open directory
listings, which is how `dew-five-year-forecast` was found. Its `community schools/` directory
holds those schools' own five-year forecasts for FY2014–FY2016 only. Nothing there is a
per-resident-district deduction.

**`reports.education.ohio.gov` is a JavaScript application.** Its `/report/finance-*` routes
return the shell, and its bundle exposes only contact and party endpoints — no finance or report
API. This is the same blocker class as `ofcc-projects`: records behind an application with no
bulk export.

**The two authenticated applications are `scholarship.ode.state.oh.us` (Scholarship Dashboard)
and `paymentdetail.ode.state.oh.us` (Payment Detail).** Both answer 200 and both serve a login.
These are a **different host** from `education.ohio.gov` and its `public.` and `reports.`
subdomains, and the first pass at this question did not reach them — recorded so the gap is not
mistaken for coverage. They are where the per-district scholarship figures most likely live, and
neither is retrievable without credentials.

**The department's own annual report points at the missing file, and the link is dead.** The
[2025 Scholarship Annual Report](https://education.ohio.gov/getattachment/About/Annual-Reports/2025-Scholarship-Annual-Report.pdf.aspx?lang=en-US)
*is* retrievable and readable, and carries statewide and program-level aggregates — Jon Peterson
alone reached 494 districts, over 80% of Ohio's, in 2024-25. For anything per-district it says a
breakdown "is available here" and links to:

    reports.education.ohio.gov/report/nonpublic-data-jon-peterson-special-needs-report      404
    reports.education.ohio.gov/report/nonpublic-data-historical-ed-choice-designated-list   404

The application's root answers 200; those two routes do not exist server-side. So the absence is
not an artifact of searching badly: a per-district route was published, is cited in a current
departmental document, and has been withdrawn.

**What is actually missing is the scholarship side, not the community school side.** Under the
Fair School Funding Plan community and STEM students are funded directly, so for them there is
no deduct to find and never will be. EdChoice, Jon Peterson, and Autism are what these dead
links covered, and they are the whole of the gap.

**The community schools payment-report page does not exist.** The sibling URL to
`Traditional-School-Districts` returns 404, and the department's CMS serves a catch-all listing
for most other finance paths.

### What would unblock it

A per-resident-district **scholarship-payment** file. Every remaining candidate needs something
other than a fetch:

- **Credentials** for `scholarship.ode.state.oh.us` or `paymentdetail.ode.state.oh.us`.
- **A public records request**, for the SFPR payment reports, which are not published as files.
- **Asking the department to restore the withdrawn route**, which is the cheapest of the three
  and the only one with evidence the file once existed in public form.

None of them is a URL, which is a different kind of blocker from the six recorded against the
connectors. A phase cannot clear it, and a phase that tries will repeat the searches above.

Implementation lands in `crates/deduction/`.
