# Fordham Institute — "Ohio House puts the brakes on Cupp-Patterson"

**Source.** Thomas B. Fordham Institute (Ohio), commentary on the House-passed version of
H.B. 96 of the 136th General Assembly. Attributed to Aaron Churchill, Fordham's Ohio research
director, published on or about 8 April 2025. [inference — the page returns no byline and no
dateline to a fetching tool; both come from search metadata rather than from the document. The
House passed H.B. 96 on 9 April 2025, so the piece is contemporaneous with the floor vote and
precedes the Senate and the conference report]
**Type.** Secondary source — policy analysis of a bill stage that did not become law.
**Location.** `fordhaminstitute.org/ohio/commentary/ohio-house-puts-brakes-cupp-patterson`.

**What it contains.** An account of the House "bridge" plan: every district and community school
held at FY2025 state aid through a guarantee, with prescribed formula increases above FY2025 cut
in half. Foundation funding increases FY2025 → FY2027 of **$179m (2.2%)** for traditional
districts, **$80m (6.3%)** for community schools, **$38m (7.6%)** for JVSDs — **$297m** and about
3% overall, against **$149m** in the executive proposal. A distributional figure by district
typology (typology 6 suburbs +4.9% against the Ohio Eight +1.3%; $118 per pupil against $100), a
table of districts with the largest FY2022–FY2025 enrollment losses, and a statewide enrollment
decline of 2.5% over three years. Three forward-looking claims: that Cupp-Patterson is on its way
out and Ohio is returning to caps and guarantees; that the House declined to stop funding empty
desks; and that it left disadvantaged pupil impact aid unfixed when **direct certification** was
available to fix it.

## The stage caveat, which here is the whole record

Every figure above belongs to the **House-passed** bill. The enacted act is a different
instrument, and [`lsc-hb96-analysis`](lsc-hb96-analysis.md)'s stage rule is what keeps the two
apart. The commentary is therefore not a description of Ohio law and must never be read as one —
it is evidence about what one chamber proposed in April 2025 and about what a school-choice-aligned
analyst wanted from the budget.

The headline did not survive conference. The bridge formula was dropped; the enacted act took the
phase-in to 83.33% and then **100%**, completing the Cupp-Patterson schedule on paper, and applied
its brake to the other dial instead — the cost inputs, held at **FY2022**. [verified —
[`legislation/hb-96-2025`](../corpus/legislation/hb-96-2025.yml)] So the piece's central claim was
overtaken, and the mechanism it named as the retreat is not the mechanism the retreat used.

## What reproduces, and against what

The commentary's FY2025 baselines are the same ones LSC carried to the greenbook, which is what
makes a stage comparison possible at all rather than an equivocation over two different years.
Its implied bases — $8,136m, $1,270m, $500m — sit on the greenbook's $8,117.9m, $1,270.3m and
$497.2m. [verified — `crates/project/fixtures/dew-greenbook.txt`, Table 2]

Placed beside the act that passed, in millions:

    school type            FY2025      House-passed        enacted     enacted - House
    traditional           8,117.9    +179   (+2.2%)   +141.5 (+1.7%)          -37.5
    community and STEM    1,270.3    + 80   (+6.3%)   +128.5 (+10.1%)         +48.5
    JVSDs                   497.2    + 38   (+7.6%)   + 69.7 (+14.0%)         +31.7
    total                 9,885.4    +297   (+3.0%)   +339.7 (+3.4%)          +42.7

**The enacted budget spent $42.7m more than the House bridge plan and gave traditional districts
$37.5m less.** The whole difference, and $37.5m besides, went to community schools and JVSDs — the
two sectors the commentary reports as the House plan's relative winners, and both did materially
better under the law than under the proposal it is criticising. [verified — the greenbook table
against the commentary's own percentages]

**The enrollment claim holds in direction and rough size, on a window this repository can see.**
Fordham reports −2.5% over FY2022–FY2025. The department's FY2027 model gives **−2.62%** over
FY2024 → FY2026, with **500 of 609** districts declining. [verified —
`crates/foundation/fixtures/fy27-department-model.csv`] Different windows, so this corroborates
rather than confirms.

**The empty-desks critique survives the enacted act intact, and is arguably stronger against it.**
The FY2027 model carries **294 districts on the temporary transitional aid guarantee at $879.0m**,
10.8% of net state funding; **263 of them are shrinking**, and they hold $786.7m of it. East
Cleveland — the commentary's own example of a district funded through a freefall — goes 1,135.9 →
1,058.3 enrolled ADM across FY2024–FY2026 and carries a **$13.7m** guarantee in the model.
[verified]

**The typology figure cannot be checked here and should not be reported as though it could.** No
typology assignment is committed anywhere in this repository, and the district-level distribution
of the House plan was an LSC simulation of a bill that died; nothing reconstructs it. Assessed
valuation per pupil is the nearest proxy the corpus holds and it is not the department's typology.
[open]

## The DPIA recommendation, which the enacted act adopted

This is the reason the entry is worth more than a bibliography line. The commentary calls the
disadvantaged pupil impact aid count "a mess" — economically disadvantaged rates inflated by
community eligibility until they no longer distinguish a poor district from one that serves free
meals to everybody — and names **direct certification**, "already used by the state", as the fix.

The enacted act did exactly that: the count becomes 75/25 and then **65/35** between the FY2025
economically disadvantaged ADM and each year's directly certified ADM. [verified —
[`formula-component/fsfp-disadvantaged-pupil-impact-aid`](../corpus/formula-component/fsfp-disadvantaged-pupil-impact-aid.yml)]
So a recommendation made against the House plan was enacted three months later, and its effect is
computable from the department's own FY2027 model, which publishes both counts per district.

**The targeting claim is correct.** Holding the statewide dollar total fixed so that only the
distribution moves, the 65/35 blend sends the Ohio Eight **+$16.07m (+11.3%, +$96 per pupil)**
against the pure economically-disadvantaged count. The districts that pay for it are the ones the
commentary described: the largest single loser is **Pickerington Local at −$3.08m**, a Columbus
suburb reporting 97.9% economically disadvantaged and 25.5% directly certified, followed by Teays
Valley, Canal Winchester and Cloverleaf on the same profile. The top valuation decile is
untouched (+0.6%, +$2 per pupil). 423 of 609 districts gain. [verified —
`crates/foundation/fixtures/fy27-department-model.csv`; the DPIA function reproduces the model's
published column to +0.019% statewide, with 607 of 609 districts inside 0.5%]

**And it is worth about a fifth of what the same provision took away.** The count change is not
distribution-neutral: it is a cut. Statewide DPIA for traditional districts falls **$84.5m (13.0%)
then $31.8m (5.6%)**, $649.2m to $532.8m before phase-in. [verified — the greenbook] The Ohio
Eight hold 30.1% of enacted DPIA, so their share of that $116.4m is roughly **−$35.1m** against
**+$16.1m** of improved targeting — a net loss near **$19m**. The commentary asked for a better
measure and received a smaller pot measured better, and it argued for the first without pricing
the second. [inference — the proportional split of the level effect is an apportionment, not a
department figure]

## Caveats

**Never merge this with the department's series.** The commentary's figures are LSC simulations of
a bill stage; the per-district figures above are the department's FY2027 calculator. The two
describe the same quantities and disagree — statewide DPIA is $532.8m in the greenbook and
$525.1m in the model, which is a vintage difference and not an error in either.

**The level-neutral comparison is a construction.** The statewide economically disadvantaged
percentage the index divides by, 0.533380310606710, is defined by R.C. 3317.02(I)(1)(a)(i) as a
computation whose denominator this corpus cannot reconstruct from the one year of calculator it
holds — the implied denominator, 1,354,592, is smaller than traditional enrolled ADM. So the
counterfactual holds the total fixed and rescales the index rather than recomputing it, which
isolates redistribution and deliberately declines to price the level. The level is taken from the
greenbook instead. [open — what the statewide percentage would have been under the prior count]

**Akron City is the one district the reproduction does not fit**, at +0.53% ($101,615). Every other
district is inside 0.5% and the median residual is −0.00004%. Why Akron differs is unestablished.
[open]

**Alignment, stated as it is for the other record.** Fordham is a school-choice-aligned
organization, and this piece reports community-school and e-school increases approvingly while
treating traditional-district increases as a cost problem. That does not make its arithmetic wrong
— none of what could be checked here was wrong — and it is the reason to check rather than adopt.
See [`fordham-base-cost-critique`](fordham-base-cost-critique.md), which sets the footing this
entry follows, and [`advocacy-literature`](../decisions/advocacy-literature.yml), which decided
that an advocacy publisher is a catalog record and never a corpus node.

**Access constraints.** Freely available. The page will not yield a byline, a dateline or its
verbatim text to a fetching tool, so both attributions above are `[inference]`. Not pinned by
digest and fed by no connector.

## Used by

- [`legislation/hb-96-2025`](../corpus/legislation/hb-96-2025.yml)
- [`formula-component/fsfp-disadvantaged-pupil-impact-aid`](../corpus/formula-component/fsfp-disadvantaged-pupil-impact-aid.yml)
