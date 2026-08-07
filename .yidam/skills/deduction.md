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

**Stub — not implemented.** The recorded blocker was `dew-foundation`, and that was wrong.
That connector is now wired with three sources and the block did not lift, because **the FY2027
funding calculator does not carry the deduction channel at all**.

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

**The community schools payment-report page does not exist.** The sibling URL to
`Traditional-School-Districts` returns 404, and the department's CMS serves a catch-all listing
for most other finance paths.

### What would unblock it

A per-resident-district deduction or scholarship-payment file. The most likely remaining homes
are the department's own SFPR payment reports, which are not published as files, and a public
records request. Neither is a URL.

Implementation lands in `crates/deduction/`.
