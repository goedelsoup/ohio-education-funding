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

**Stub — not implemented.** Blocked on `dew-foundation`. Implementation lands in
`crates/deduction/`.
