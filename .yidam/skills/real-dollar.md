---
name: real-dollar
description: Normalize a nominal Ohio funding series to constant dollars before any cross-period comparison; mandatory, not optional
---

# Procedure: real-dollar

**Purpose.** Turn a nominal series into one whose values can be compared. Required before *any*
claim that spans fiscal periods, including ones phrased as questions rather than as numbers.

**Composes.** [`deflate`](deflate.md).

## Steps

1. Identify the fiscal-year alignment of the series. Ohio fiscal years end 30 June; the deflator
   uses the June observation of CPI-U all items, `CUUR0000SA0` period `M06`.
2. Choose and **state** a base year. A real series without a named base is as unreadable as a
   nominal one.
3. Deflate through [`crates/deflate`](../../crates/deflate/), which is verified against the
   Bureau's published file rather than against a table somebody typed.
4. Report nominal and real together. Dropping the nominal series makes the figures
   irreconcilable with every published source, which all quote nominal.

## Why this is mandatory here rather than good practice

**H.B. 920 is invisible without it.** A district at the reduction-factor ceiling has flat nominal
revenue, which reads as stability. Deflated, it is a decline every year, and that decline is the
entire mechanism the corpus exists to describe. A nominal series does not understate the effect;
it shows the opposite of it.

The same is true of the base cost. $5,283 in FY2006 against $8,241.61 in FY2027 is a 56% nominal
increase and a much smaller real one, and which of those a reader takes away decides whether they
think Ohio has increased school funding.

## Preconditions and refusals

- **Refuse to deflate across a regime boundary without marking it.** Deflation makes two numbers
  comparable in units; it does not make them comparable in meaning. Base cost before and after the
  Fair School Funding Plan is computed two different ways — one legislated, one measured — and a
  smooth line across FY2021 implies a continuity that does not exist.
- **Refuse to deflate a per-pupil figure without fixing the denominator.** Two deflated series on
  different pupil counts are still not comparable.
- CPI-U is a consumer basket. School costs are majority salaries, and an education-specific
  deflator would tell a different story; ECI was approved as a source at genesis and is not
  retrieved. Say which index was used. `[open]`
