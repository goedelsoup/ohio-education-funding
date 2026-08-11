# OCG White Paper No. 015 — Has Ohio Been Defunding Public Education?

**Source.** Ohio Common Ground Research Center, "Has Ohio Been Defunding Public Education? A
Two-Decade Analysis of School Funding, Enrollment, Spending, and the Claims Made About Them —
With Selected Updates Through FY2025." White Paper No. 015, Tracking ID RL-2026-023, Version 1.0,
August 11, 2026.
**Type.** Secondary source — analysis over primary data.
**Location.** `ohiocommonground.com`.

**What it contains.** Five commonly repeated claims about Ohio school funding, each tested
measure by measure against four deliberately separated funding "universes" (all-funds NCES, state
appropriations, foundation-delivered, scholarship payments). Its central device is a table of
**seven standards** for the word "defunding" — nominal reduction, real reduction, per-pupil
reduction, declining state share, purchasing-power erosion, unfunded statutory formula, and
funding relative to obligations — with Ohio's answer under each.

**This is the strongest of the three OCG publications this corpus holds.** Keeping the universes
apart and never combining them, assigning a verdict per measure rather than collapsing to a
grade, and returning *Insufficient Evidence* on the "bankrupting" claim rather than stretching to
an answer are all correct choices. Section 8 also avoids the error this corpus recorded against
[White Paper 013](ocg-white-paper-013.md): it reports statewide NAEP alongside funding with an
explicit no-causation caveat rather than running a district cross-section.

**Full corroboration on the Auditor series.** Every figure it draws from the Ohio Auditor's
Longitudinal School Finance Study matches this corpus's independently held values, which come
from the same source through the [`deflate`](../skills/deflate.md) calculator: $7,065 → $15,314,
+116.8% nominal and **+26.1% real**; $14,493 excluding relief, +19.4%; total operating
expenditure $12.97B → $25.78B; enrollment −8.3% from 1.84M to 1.68M; CPI +71.9%; instruction
+25.7% and support services +28.2% real. [verified — see
[`crates/deflate/tests/ohio_epp_real_series.rs`](../../crates/deflate/tests/ohio_epp_real_series.rs)]

## Three findings the corpus adds

**1. The real series is not monotone, and the endpoint framing conceals a real decline.** Figure
1 plots the *nominal* series, which rises smoothly. Deflated year by year, real per-pupil
spending **fell about 7% from FY2010 to FY2014** — $15,226 to $14,173 in constant FY2022 dollars
— leaving FY2014 indistinguishable from FY2006 and not recovering to the FY2010 level until
FY2018. [verified]

The seven-standards table answers "real reduction" and "per-pupil reduction" with **No**. That is
correct for FY2000→FY2022 and false for FY2010→FY2014, which is plausibly the period a
"they defunded us" speaker has in mind. The paper's structural thesis — that the disagreement is
definitional — is right and incomplete: it is also **periodic**. Two speakers can choose
different windows on one series and both be correct, and no column in the table surfaces that.

The interior rows rest on chart labels read to about $100; the decline is $1,053, and a test
perturbs both endpoints in the direction that would erase it and finds it intact.

**2. Real per-pupil spending peaked in FY2020, not FY2022.** $15,747 against $15,314 — the FY2022
endpoint sits **$433, or 2.7%, below the real peak**, because relief arrived while prices were
low and was then eroded by FY2021–22 inflation. [verified] The "record funding" verdict is marked
Supported on the *real per-pupil* row; that is true against FY2000 and not true as a record.

**3. At full phase-in the "unfunded statutory formula" standard does not resolve — it inverts.**
Section 5 reports FY2024: $741.8M below the formula, 180 districts drawing $181.2M in guarantee.
The department's own FY2027 model — the terminal year, formula at 100% — shows **294 of 609
districts (48.3%), holding 54.1% of Ohio's students, funded by the
[guarantee](../corpus/formula-component/temporary-transitional-aid-guarantee.yml) rather than the
formula, totalling $878,974,300.** [verified] Roughly fivefold growth in guarantee dollars
arriving *at* completion of the phase-in.

The guarantee is anchored to **FY2020 and has never been re-based**; for guaranteed districts the
FY2027 formula produces a median 67.8% of that baseline. The paper's Future Research asks to
"track FSFP phase-in through FY2027 and whether the formula is funded to its calculated amount."
Full phase-in does not put districts on the formula; it leaves half the state on a hold-harmless
anchored to a year the formula was not run.

## The trough, now located district by district

The paper's per-pupil-reduction row reads **No** for FY2000-FY2022. Over FY2010-FY2013 the
district panel shows **81% of Ohio's comparable districts spending less per pupil in real terms**,
median -5.6%. [verified] It was not a few districts moving an average, and nothing about a
district — its state, federal or local revenue share, its later poverty or property wealth —
predicts how far it fell. A general contraction, not a targeted one.

Extending the same panel back to FY2009 also enlarges the paper's Section 4 finding on its own
terms. The state share of district revenue falls **11.7 points** from FY2009 to FY2022 on the
F-33 comparable-district basis, against the 6.2 points the paper reports for FY2000-FY2022 on the
wider all-funds basis. Different universes, not directly comparable — but the direction is the
paper's and the recent decline is steeper than a two-decade endpoint pair conveys. [verified]

## The window it could not reach, and it points the other way

The all-funds series ends at FY2022 and Future Research asks for an extension. This corpus holds
actual district receipts for 660 districts, **FY2020–FY2025**, over which CPI rose 25.1%:
[verified — see
[`crates/project/tests/finances_and_the_guarantee.rs`](../../crates/project/tests/finances_and_the_guarantee.rs)]

| | nominal up | real up | median real change |
|---|---|---|---|
| unrestricted state aid | 491/658 | **196/658** | **−11.4%** |
| total revenue | 637/659 | 259/659 | −2.8% |
| total expenditure | 632/658 | 380/658 | +3.4% |
| property tax | 637/659 | 345/659 | +0.9% |

Aggregate unrestricted state aid: **$7.82B → $7.89B nominal (+1.0%), −19.3% real**, with 70.2% of
districts losing real ground. On the paper's own "real reduction" standard applied to state aid
over the six most recent observed years, the answer is **Yes**. What districts spend held its
real value; what the state sent them did not, and the difference was made locally.

The mechanism is recorded on the guarantee node: it is a **nominal** floor, so a district it
protects loses purchasing power annually by construction, with nothing in the formula recording
that it happened.

## Corroboration on the voucher mechanism

Section 6's load-bearing claim — that post-FSFP scholarships are direct state payments rather
than deductions from resident-district aid — is independently confirmed. This corpus searched
district payment reports for a deduction channel and found none; the transfer lines are too small
and run in both directions. [verified — see
[`crates/project/tests/the_voucher_channel_is_absent.rs`](../../crates/project/tests/the_voucher_channel_is_absent.rs)]

One standing caution from [`skills/deduction`](../skills/deduction.md): a series spanning the
FY2022 transition without marking it shows foundation payments rising for reasons unrelated to
the formula. The paper's own comparisons are post-FSFP on both sides and clean; a reader spanning
FY2000–FY2025 will not necessarily be.

## Section 9, now half-checked

The FY2024 F-33 survey is held and the cross-state table splits in two. [verified — see
[`crates/dispersion/tests/census_f33_fy2024.rs`](../../crates/dispersion/tests/census_f33_fy2024.rs)]

**The revenue mix reproduces, and it is the part the argument rests on.** Ohio's shares come out
state 35.1% / local 53.3% / federal 11.6% against the published 34.6 / 53.6 / 11.8; Indiana and
Kentucky land within half a point on every line. Ohio's local reliance sits more than nine points
above the national figure and above all three of its western and southern neighbours. The
paper's Section 9 claim is confirmed.

**The per-pupil dollars do not reproduce, and the corpus cannot say why.** Aggregating the
individual-unit file gives Ohio $16,610 against a published $17,257, and the errors scatter
rather than shift: Indiana +0.5%, Kentucky +1.1%, Pennsylvania +3.2%, Ohio −3.7%, Michigan
−9.8%. A uniform bias would point at a spending definition or a unit filter and could be
corrected; a state-specific one cannot be from these rows. The Bureau publishes its state
per-pupil figures from summary tables built on something the unit file does not carry.

That is recorded as a limit on this corpus, **not** as an error in the paper, which quotes a
published Census figure correctly. It does mean nobody should rebuild a state per-pupil
comparison from the unit file and expect the published numbers. [open]

## What this corpus still cannot check

NAEP (Section 8), the FY2014–FY2025 scholarship payment series (Section 6), and the LSC
appropriation history remain outside the corpus's coverage and are recorded as unverified rather
than accepted.

## Used by

- [`metric/per-pupil-operating-expenditure`](../corpus/metric/per-pupil-operating-expenditure.yml)
- [`formula-component/temporary-transitional-aid-guarantee`](../corpus/formula-component/temporary-transitional-aid-guarantee.yml)
