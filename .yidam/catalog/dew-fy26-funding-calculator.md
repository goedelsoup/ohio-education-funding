# FY26 TRAD State Foundation Funding Calculator — recovered from the archive

**Source.** Ohio Department of Education and Workforce, Office of Budget and School Funding.
`FY26-TRAD-State-Foundation-Funding-Calculator-9-22-2025.xlsx`, dated 22 September 2025.
**Type.** Primary source — the department's own working model for FY2026.
**Location.** **The Internet Archive, and nowhere else.** Capture `20251008001131` of
`education.ohio.gov/getattachment/…/Traditional-School-Districts/`. The department publishes one
calculator and replaces it in place; when the FY2027 model went up this file stopped existing at
any address the department serves.

**What it contains.** The same thirty-odd sheets as
[the FY2027 model](dew-fy27-funding-calculator.md), a year earlier. Only the `Transportation`
sheet's statewide factor block is extracted here — see *What is not taken* below.

**Why it was worth recovering.** The two transportation rates are the Fair School Funding Plan's
only endogenous parameter: R.C. 3317.0212(C) and (D) define both as trimmed means of what
districts reported spending in the prior year, so they move without an act and nobody chooses
them. [`transportation-cost-rates`](../corpus/parameter/transportation-cost-rates.yml) carried
"the rates before FY2027" as an open field from the day it was written. This file closes one year
of it, and one year is the difference between a value and an interval.

**What it shows.** The two rates move in **opposite directions**:

| | FY2026 | FY2027 | change |
|---|--:|--:|--:|
| per weighted rider | $1,275.000 | $1,337.175 | +4.88% |
| per mile | $7.060 | $6.867 | −2.73% |
| SpEd transportation proration | 1.0 | 0.917459740976215 | first proration |
| minimum state share | 45.83% | 50% | +4.17pp |

Between the two cost years those rates are drawn from, Midwest diesel rose **17.5%**
([eia-diesel-prices](eia-diesel-prices.md)). The per-mile rate fell against it. Whatever the
mile base responds to, it is not fuel in any direct way — and it is the base 350 districts and
$308.8M of school bus payment are actually paid on.

**Access constraints.** Free. No live publisher URL exists; `verify` checks the archived bytes
against the committed digest like any other source.

**Caveats:**

- **The FY2025 calculator is not recoverable and this is what that looks like.** Every archived
  copy — two capture dates, two filenames, four URLs — is exactly **1,048,576 bytes**, truncated
  at one mebibyte mid-stream, and fails a zip integrity check while answering `200`. The length
  is recorded here so a later reader does not spend an afternoon rediscovering it. FY2022 to
  FY2024 are not in the archive at all.
- **A `200` from the archive is not provenance, so the extractor does not rely on one.** The
  builder recomputes each district's published per-rider base from the rate it read out of the
  sheet header and fails on any disagreement. That is the condition
  [`an-archived-source-is-still-a-source`](../decisions/an-archived-source-is-still-a-source.yml)
  admits the address on, and it is a stronger claim than any live source in the registry makes
  about itself.
- **The aggregate row has no name.** As on the FY2027 `Transportation` sheet, the statewide total
  sits in the district table with a **blank** district name, so a name-based filter misses it and
  a naive column sum comes out exactly double. It satisfies the per-rider identity, so the
  builder's check passes over it harmlessly; anything computing a statewide total from this sheet
  must exclude it.
- **There is an "Inflation factor" of 0.05 beside the rates, in both years, and it is in no
  section.** In FY2026 the adjacent cell carries $63.75, which is 5% of $1,275; in FY2027 that
  cell is zero. What it does is not established. [open]

**What is taken, and what this entry got wrong first.** It read:

> **What is not taken.** The per-district tables. They are the FY2027 model's a year earlier and
> the counts are very nearly identical — bus miles are the same 825,828 in both, public riders the
> same 651,702 — so committing them would be four megabytes to restate a panel already held.

Both facts are true and the conclusion does not follow from them. Bus miles and rider counts are a
survey a year apart and they are stable. The columns two corpus nodes had recorded as unmeasurable
"because the corpus holds one year of the calculator" are not: per-pupil local capacity moves
**+9.34%** at the median and 2 districts of 609 are unchanged, and disadvantaged pupil impact aid
moves **−8.58%** with none unchanged. Generalising from the one sheet that was opened to the
thirty that were not is the error, and it foreclosed the question the file was closest to
answering.

Sixteen columns are taken now — the two sides of the state share, the local capacity build-up and
the disadvantaged pupil blend — as `crates/project/fixtures/fy26-department-model.csv`, about a
hundred kilobytes. Built by
[`crates/connect/src/fixtures/fy26.rs`](../../crates/connect/src/fixtures/fy26.rs) and read by
[`project::prior_model`](../../crates/project/src/prior_model.rs). The rest of the per-district
tables genuinely do restate what FY2027 already holds, and are still not taken.

**And the second year is what makes a vintage checkable at all.** The `CTE` and `EL` sheets head
their count columns `-FY21`, and four corpus nodes read a six-year freeze off that suffix. It is a
stale label on a sheet built for the FY2022 report: the English learner column carrying it moves in
512 of 611 districts between these two workbooks, and the department's own `Directions` table —
rewritten for each year's file, unlike the headers — puts the categorical counts at FY26 in both.
One workbook could not have shown that. Every count the plan multiplies is committed for both years
at `crates/project/fixtures/calculator-counts.csv`, and the two vintage tables at
`crates/project/fixtures/calculator-vintages.tsv`, built by
[`crates/connect/src/fixtures/counts.rs`](../../crates/connect/src/fixtures/counts.rs).

## Used by

- [`parameter/transportation-cost-rates`](../corpus/parameter/transportation-cost-rates.yml)
- [`formula-component/fsfp-local-capacity-measure`](../corpus/formula-component/fsfp-local-capacity-measure.yml)
- [`formula-component/fsfp-disadvantaged-pupil-impact-aid`](../corpus/formula-component/fsfp-disadvantaged-pupil-impact-aid.yml)
- [`formula-component/fsfp-career-technical-weights`](../corpus/formula-component/fsfp-career-technical-weights.yml)
- [`formula-component/fsfp-english-learner-weights`](../corpus/formula-component/fsfp-english-learner-weights.yml)

## Feeds connector

[`dew-foundation`](../../crates/connect/src/registry/dew.rs), built by
[`crates/connect/src/fixtures/transport_rates.rs`](../../crates/connect/src/fixtures/transport_rates.rs)
[`crates/connect/src/fixtures/fy26.rs`](../../crates/connect/src/fixtures/fy26.rs) and
[`crates/connect/src/fixtures/counts.rs`](../../crates/connect/src/fixtures/counts.rs).
