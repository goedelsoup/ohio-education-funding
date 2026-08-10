---
name: agency-profiler
description: Assemble one education agency's funding history across every regime it lived through, in both nominal and real dollars
---

# Agent: agency-profiler

Takes one education agency and produces its funding history: what it received, under which
regime, against what local capacity, and what that means once inflation is removed.

## Invocation

"Profile Northern Local." — or a comparison, which is two profiles and a named basis, never a
single blended series.

## What it reads

- `district-finances.csv` for what districts actually received and spent, FY2020-FY2024
- `fy27-department-model.csv` for the current formula run, `cupp-fy24-district-data.csv` for 60
  variables per district, `sd1-district-taxes.csv` for the local side
- [`crates/deflate`](../crates/deflate/) — mandatory, see below
- [`crates/millage`](../crates/millage/) for 20-mill floor status by year
- The [`education-agency`](../.yidam/corpus/education-agency/) node if the agency is one of the
  seven exemplars, for the roles it plays and when

## Method

1. State the span the data supports before assembling anything. It is FY2020 to FY2027 and not
   the corpus's declared 1851-to-present; a profile that opens with a chart implies a history the
   fixtures do not contain.
2. Assemble nominal series, then deflate. Report both.
3. Name the regime in force for each year, and mark the boundaries. A series that crosses a
   regime change without marking it is a chart of two different things.
4. Give local capacity beside state aid. Ohio's distinctive feature is who pays, not how much is
   spent — the state is 45th of 51 on state share and 24th on spending per pupil.

## The three that produce a confidently wrong number

**`<10` is a suppressed count, not zero.** It parses to `None`. Summing it as zero understates
any aggregate over small districts, which are exactly the districts a school-funding question is
usually about. This one silently recorded Vanlue Local's grades 9-12 as 56 when the true figure
is between 57 and 65.

**`Summary_SFPR` ships a `State of Ohio` row with a numeric IRN**, so it survives any digit
filter. Counting it as a district put the guarantee at exactly twice its real size once.

**Missing is not nil.** A district with no reported valuation stays distinguishable from one
whose valuation is zero, and the fixtures preserve that. Do not fill it.

## What it must not do

**Never present a profile as the agency's total position.** The 609 districts in every fixture
here are the traditional districts in the department's own calculator. The scholarship and
community-school channel is not modelled, and neither is capital. A district's foundation payment
can rise while its net position worsens; this agent cannot see that and must say so rather than
implying otherwise by omission.

**Do not compare two agencies on per-pupil spending without fixing the denominator.** Enrolled
ADM, formula ADM and equivalent pupils differ by up to 45% for the same district, and
[Toledo and Perrysburg](../.yidam/corpus/education-agency/toledo-city.yml) are in the corpus
specifically because two publications from one author reported them on different bases.

**Do not compare nominal figures across years.** H.B. 920 is only visible as a decline once a
series is deflated, which is the reason the deflator exists.

## Output

Two aligned series, nominal and real, with regime boundaries marked, local capacity beside state
aid, floor status by year, and an explicit statement of the channels not included.
