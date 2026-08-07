# OCG Ground Truth Fact-Check RL-2026-021 — Toledo–Perrysburg Special-Needs Spending

**Source.** Ohio Common Ground Research Center, *Ground Truth* fact-check, "Do Ohio's Urban
Districts Spend Far More on Special-Needs Students? A Toledo–Perrysburg Comparison." Tracking ID
RL-2026-021.
**Type.** Secondary source — analysis over primary data.
**Location.** `ohiocommonground.com`.

**What it contains.** A verdict of FALSE on a circulating claim that Toledo Public Schools
devotes ~45% of spending to special-needs or student-support services against 14–16% for
Perrysburg. Reports, for FY2024–25: student-support share 8.1% / 8.5%, special-education
instruction share 14.5% / 14.4–14.7%, special-education instruction per pupil $2,766 /
$2,032–$2,363, and total operating per pupil $20,805 / $14,632.

**The verdict is correct and the sourcing is real.** Both figures this corpus can check reproduce
exactly from the department's own FY2025 Expenditure Expanded List: student support 8.1% and
8.5% (the `Pupil Support` function over operating expenditure), and total operating per pupil
$20,805 and $14,632 (operating expenditure over **unweighted** ADM). [verified — see
[`crates/dispersion/tests/expenditure_functions_fy25.rs`](../../crates/dispersion/tests/expenditure_functions_fy25.rs)]

**Three findings from reviewing it.**

**1. The interpretive gloss inverts under the right denominator.** The document says Toledo
"does spend somewhat more per pupil on special education … consistent with a modestly heavier
student-need profile." Toledo's disability share is **21.9%** against Perrysburg's **11.3%** —
nearly double, not modest — while its special-education spending per *total* pupil is only
1.17–1.36 times as high. Per student with a disability the comparison reverses: roughly
**$12,600** in Toledo against **$18,000–$20,900** in Perrysburg. [inference — the
special-education dollars are the fact-check's own, from audited statements this corpus does not
hold, so this is an order-of-magnitude normalisation rather than an audited cost. A 1.94x
population difference against a 1.36x spending difference leaves no room for the sign to return.]

This is the corpus's denominator rule arriving from a new direction: not a need-weighted pupil
count this time, but a special-education figure divided by a population that is mostly not
receiving the service.

**2. The table's four rows are not on one basis.** $2,766 ÷ $20,805 = 13.3%, not the 14.5% in the
row above; $2,032–$2,363 ÷ $14,632 = 13.9%–16.1%, which brackets rather than matches the reported
14.4–14.7%. The footnote does say the bases differ — General Fund for Toledo, General Fund and
all-funds for Perrysburg — but a reader performing the obvious division gets a different number
than the table states, in a document whose purpose is correcting someone else's arithmetic.
[verified]

**3. It silently contradicts the same author's White Paper 013 on the same district.** For Toledo
City, FY2025: this fact-check reports **$20,805** per pupil;
[White Paper 013](ocg-white-paper-013.md) analyses the report card's **$14,312** per *equivalent*
pupil. A 45% spread on one district in one year, three weeks apart, from one publisher, with
neither document noting the other's basis. [verified]

Worth recording plainly: this fact-check uses the **better** denominator. The headcount basis is
the one this corpus concluded was the cleaner instrument. The author has switched to it without
remarking on the switch, which is the problem — a reader of both publications sees Toledo
spending two very different amounts and has no way to reconcile them.

**What it does not claim, correctly.** It disclaims any judgment about adequacy, efficiency, or
sustainability, and notes that the special-instruction function excludes related services and
special-education transportation, so it understates total special-education cost symmetrically
for both districts. Both caveats are accurate and neither is undercut by the above.

**Caveats on using it:**

- Secondary. The special-education figures come from district audited financial statements this
  corpus does not hold and has not checked.
- Two districts. Nothing here establishes anything about the state distribution; the corpus's own
  function fixture puts Toledo above and Perrysburg below the statewide median of $16,289.
- The comparison the file actually supports and the fact-check did not make is function-level:
  Toledo spends $6,173 more per pupil and a markedly smaller share of it on instruction (51.3%
  against 62.6%) and classroom instruction (63.4% against 73.1%). [verified]

## Used by

- [`education-agency/toledo-city`](../corpus/education-agency/toledo-city.yml)
- [`education-agency/perrysburg-exempted-village`](../corpus/education-agency/perrysburg-exempted-village.yml)
- [`metric/expenditure-per-equivalent-pupil`](../corpus/metric/expenditure-per-equivalent-pupil.yml)
