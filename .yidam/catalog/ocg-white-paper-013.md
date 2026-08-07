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

**Arithmetic audit — passes.** Every internal reconstruction this corpus could perform
reconciles to the published value: quintile means × counts return the reported spending mean
($13,223.7 against $13,224) and Performance Index mean (87.70 against 87.7) from both the
spending and the performance quintile tables; the Fisher-z interval, the three p-values, and the
partial correlation all reproduce from the stated coefficients and n = 607; Table 3's components
sum to its totals in every row. Solving the covariance identity
`r_t·σ_t = r_f·σ_f + r_sl·σ_sl` jointly with `σ_t² = σ_f² + σ_sl² + 2r_fsl·σ_f·σ_sl` implies
σ(federal) ≈ $483 and σ(state-and-local) ≈ $2,639, both consistent with the published quantiles.
The reported correlation set is mutually consistent. [verified — recomputed from the published
tables; the underlying DEW files have **not** been independently pulled]

**Independent corroboration of a corpus figure.** Its spending dispersion, CV = $2,660/$13,224 =
**0.201**, matches the coefficient of variation of **0.202** that
[`crates/dispersion`](../../crates/dispersion/src/lib.rs) computes over the FY2024 Cupp Report —
different year, different source file, different denominator, same dispersion. This is the first
external check on that statistic. [verified]

**Where this corpus departs from it.** The paper's central claim is that the association is
near zero. The corpus's position, recorded at
[`metric/expenditure-per-equivalent-pupil`](../corpus/metric/expenditure-per-equivalent-pupil.yml),
is that the near-zero coefficient is substantially a product of the need-weighted denominator
inside the spending variable, that "unadjusted" is therefore true of the model and not of the
measure, and that the paper's own enrollment-weighted result (−0.149) is the one more consistent
with what the corpus knows about how Ohio spending is distributed. The paper's stated limitation
— "no need-adjusted model in this version" — understates the issue: there is a need adjustment,
on one side, running in a known direction.

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
