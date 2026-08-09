# Ohio's sexennial reappraisal and triennial update calendar

**Source.** Ohio Department of Taxation, Tax Equalization Division, *Year of Sexennial
Reappraisal and Triennial Update for Ohio's 88 Counties, 2024–2029*.
**Type.** Primary source — the division that approves county valuations, publishing its own
schedule for doing so.
**Location.**
`dam.assets.ohio.gov/image/upload/tax.ohio.gov/real_estate/yearofsexennialreappraisalandupdate-2024-2029.pdf`.
One page.

**What it contains.** For each of the six years 2024 through 2029, the counties undergoing a full
**sexennial reappraisal** and the counties undergoing a **triennial update**. Every county appears
exactly twice across the six years, three years apart, alternating between the two.

**Why this is here.** Because the charge-off was applied to **recognized valuation**, and this
calendar is the whole of what decides a district's recognized valuation in any given year. See
[local share charge-off millage](../corpus/parameter/local-share-charge-off-millage.yml) for what
the corpus previously believed recognized valuation to be, which was a different mechanism.

The property that makes it useful is one the published table does not state: **every county has
exactly one valuation event in TY2022–TY2024**. That falls out of the three-year cadence, and it
means a four-year window of [Table SD-1](dot-sd1-school-district-taxes.md) gives each district one
reappraisal and two quiet years — enough to separate a revaluation from ordinary growth without
any second source.

**How the pre-2024 years are derived.** The table starts at 2024 and the corpus needs TY2022 and
TY2023. They are derived rather than retrieved, and exactly: the cycle is reappraisal, update
three years later, reappraisal three years after that, so a county listed for reappraisal in 2026
updated in 2023, and one listed for update in 2025 was reappraised in 2022. The published table
validates the rule on its own face — its 2024 reappraisal counties are precisely its 2027 update
counties.

**Not committed as a fixture.** No PDF text extractor exists in this workspace and 88 rows do not
justify growing one. The calendar is carried as constants in
[`regime_diff::recognized_valuation::CYCLE`](../../crates/regime-diff/src/recognized_valuation.rs),
on the same grounds as the charge-off rate series: a small, stable, citable table is a parameter
rather than a data file.

**And it is checked against data that would expose a transcription error.** A hand-typed table
invites a typo, so the test does not read the transcription back to itself. It takes each county's
real property value from Table SD-1 across four tax years and asserts the largest year-over-year
jump falls in the year the calendar names. **All 88 pass**, against a median event-year jump of
28.6% and a median quiet year of 1.5% — a misassigned county cannot hide behind a gap that wide.

**Vintage risk.** The schedule is republished each cycle, and a county can be moved by the Tax
Commissioner. The digest manifest is not the detector here, because the file is not fetched into
the cache; the empirical test is, and it will fail loudly if the calendar and the abstract stop
agreeing.

## Used by

- [`parameter/local-share-charge-off-millage`](../corpus/parameter/local-share-charge-off-millage.yml)
- [`decisions/recognized-valuation`](../decisions/recognized-valuation.yml)
