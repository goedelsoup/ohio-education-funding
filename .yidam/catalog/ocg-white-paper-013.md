# OCG White Paper No. 013 — Does Per-Pupil Spending Track Academic Performance?

**Source.** Ohio Common Ground Research Center, White Paper No. 013, "Does Per-Pupil Spending
Track Academic Performance? An Unadjusted, Single-Year Cross-Sectional Analysis of Expenditure
per Equivalent Pupil and Performance Index Across Ohio's Traditional Public-School Districts,
2024–2025." Tracking ID RL-2026-020, Version 1.0. Dublin, Ohio, August 7, 2026.
**Type.** Secondary source — analysis over primary data, with a published replication package.
**Location.** `ohiocommonground.com`.

**What it contains.** A district-level cross-section of 607 rated traditional districts joining
the [report-card Performance Index](dew-report-card-achievement.md) to
[report-card expenditure per equivalent pupil](dew-report-card-spending.md) by IRN, with
enrollment from the FY2025 Expenditure Expanded List. Reports Pearson and Spearman coefficients,
Fisher-z intervals, spending and performance quintiles both unweighted and enrollment-weighted,
and a seven-row sensitivity table.

**Headline figures, as published:**

| Quantity | Value |
|---|---|
| Total spending per equivalent pupil vs Performance Index | Pearson −0.016 (95% CI −0.095 to +0.064; p = 0.703); Spearman +0.048 |
| Same, weighted by unweighted ADM | Pearson −0.149 |
| Federal expenditure per equivalent pupil vs Performance Index | Pearson −0.558; Spearman −0.694; partial (holding state-and-local) −0.556 |
| State-and-local expenditure per equivalent pupil vs Performance Index | Pearson +0.086 (p = 0.034) |
| Enrollment vs spending per equivalent pupil | Spearman −0.366 |
| Spending distribution, 607 districts | median $12,856, mean $13,224, SD $2,660 |
| Performance Index distribution | min 53.1, median 88.2, mean 87.7, max 112.8, SD 10.9 |

**Replicated against the primary sources — every figure reproduces.** The corpus now retrieves
the same three DEW files and recomputes the paper's results from them. To the precision the paper
displays, all of it lands: −0.016, Spearman +0.048, federal −0.558, state-and-local +0.086,
enrollment-weighted −0.149, enrollment against spending ρ −0.366, median $12,856, mean $13,224,
and the Performance Index at median 88.2, mean 87.7, sd 10.9, range 53.1 to 112.8. Pinned in
[`crates/dispersion/tests/report_card_2425.rs`](../../crates/dispersion/tests/report_card_2425.rs).
[verified]

This is an unusually clean replication and it should be said plainly: the paper's arithmetic is
correct, its sources are cited accurately enough to find and reproduce, its sensitivity table
holds, and its stated caveats are honest. The disagreement below is about measurement, not
competence.

**Independent corroboration of a corpus figure.** Its spending dispersion, CV = $2,660/$13,224 =
**0.201**, matches the coefficient of variation of **0.202** that
[`crates/dispersion`](../../crates/dispersion/src/lib.rs) computes over the FY2024 Cupp Report —
different year, different source file, different denominator, same dispersion. This is the first
external check on that statistic. [verified]

**Where this corpus departs from it, now measured.** The paper's central claim is that the
association is near zero. It is near zero *on the denominator the paper chose*. The department's
FY2025 Expanded List publishes one operating-expenditure total against both a weighted and an
unweighted pupil count, and correlating the Performance Index against the same dollars gives
**−0.015** on the first and **−0.337** on the second. [verified] Same districts, same year, no
covariates — only the divisor changes.

"Unadjusted" is therefore true of the paper's model and false of its measure. The stated
limitation, "no need-adjusted model in this version," understates it: there is a need adjustment,
inside the independent variable, on one side only, pointing the way that flattens the result. See
[`metric/expenditure-per-equivalent-pupil`](../corpus/metric/expenditure-per-equivalent-pupil.yml).

**Two of its own conjectures, tested.** The paper hypothesised that its federal-spending result
was a poverty signal and said it could not test that directly. Holding the economically
disadvantaged share constant, federal per-pupil falls from −0.558 to **−0.158** — the hypothesis
was largely right. And the variable it was standing in for is far stronger than anything the
paper reports: disadvantage against the Performance Index is **−0.846**, 71.6% of the variance.
[verified]

**The measure it names in Future Research answers the question differently.** The paper lists
"add other outcome measures, including the Progress (value-added) component." Done: against
Progress, holding disadvantage constant, per-pupil spending correlates **+0.146**, where against
the Performance Index it correlates −0.125. Its own proposed next step reverses the sign of its
subject. Its other proposed step — lagged rather than same-year spending — does not help either:
FY2024 spending tracks the *prior* year's Index (−0.426) slightly better than the following
year's (−0.388), so there is no forward-directed signal to lag into. [verified]

**Its priority next step has been taken, and it reverses the paper's subject.** The paper names
"a need-adjusted, multivariable descriptive model relating Performance Index to per-pupil spending
while accounting for economically disadvantaged, English-learner, and disability shares, district
enrollment, typology, and local wealth" as its first Future Research item. That model now exists
in this corpus, missing only typology. The standardised spending coefficient is **−0.073** on the
Performance Index and **+0.209** on the Progress component. [verified] Its own proposed test
returns opposite signs depending on which of Ohio's two published outcome measures is used, and
the paper analysed only one of them.

**One correction of description.** The paper reads state-and-local spending across performance
quintiles as "moving in the opposite direction, from $12,403 to $13,324." That row is flat within
$200 across four quintiles and steps $984 at the top. It is a step at the wealth end, not a
gradient.

**Caveats on using it:**

- Secondary. Every figure above is as-published and reaches DEW only through this paper.
- Single year, single outcome measure, no covariates. The paper says so repeatedly and does not
  overclaim; it draws no causal conclusion and makes no recommendation.
- Its district count (607) is not the corpus's (606 for FY2024 Cupp, 609 for the FY2027 model).
- Its enrollment total (~1.47M, FY2025) exceeds the corpus's FY2024 enrolled ADM total
  (1,439,473) while Ohio enrollment is falling, so the two ADM definitions are not the same
  quantity. [open]

## Used by

- [`metric/performance-index`](../corpus/metric/performance-index.yml)
- [`metric/expenditure-per-equivalent-pupil`](../corpus/metric/expenditure-per-equivalent-pupil.yml)
