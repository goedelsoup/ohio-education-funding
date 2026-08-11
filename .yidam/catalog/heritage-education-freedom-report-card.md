# Heritage Foundation — Education Freedom Report Card (2023, 2nd edition)

**Source.** The Heritage Foundation, *2023 Education Freedom Report Card: State Rankings for
Parents*, second edition, highlights booklet. Edited by Lindsey M. Burke, Jay P. Greene, Jonathan
Butcher, Jason Bedrick and Madison Marino.
**Type.** Secondary source — advocacy scorecard ranking states on school choice, regulation,
transparency and spending.
**Location.** `static.heritage.org/education/education-freedom-reportcard/highlights-booklet/2023/`.

**What it contains.** A rank per jurisdiction overall and in each of four categories —
Education Choice, Teacher Freedom, Transparency, Return on Investment — "that encompass more than
two dozen discrete factors". 51 jurisdictions are ranked, the 50 states and the District of
Columbia; Oregon is 51st.

**Ohio's result, 2023 edition:** [verified]

| Category | Ohio's rank of 51 |
|---|--:|
| **Overall** | **29** (down from the prior edition) |
| Education Choice | 12 |
| Transparency | 37 |
| Teacher Freedom | 47 |
| Return on Investment | 47 |

**Return on Investment is a spending category, and lower spending scores better.** This is the
fact that governs how the document may be cited here, and it is explicit rather than inferred.
The category's framing is "education liberty includes the freedom of taxpayers to keep more of
their money"; its four variables are per-pupil spending (nominal, and adjusted by Bureau of
Economic Analysis regional price parities for 2018), NAEP points per dollar spent,
teacher-to-non-teacher ratio, and unfunded pension liability as a share of state GDP. The
direction is stated in the booklet's own advice to another state: "Iowa can improve its ROI
ranking by **lowering per-pupil spending**, stopping growth in non-teaching staff, and lowering
its unfunded teacher pension liabilities." The booklet's own summary card relabels the category
"SPENDING RANK".

So Ohio's 47th on Return on Investment is substantially a statement that Ohio spends a lot per
pupil. It is not an efficiency finding, and it is not evidence about adequacy in either
direction. A reader who took the four category ranks as four independent assessments of Ohio's
schools would have that one backwards.

**The choice score rewards the absence of regulation on participating private schools.** The
Private-School-Choice Program Design variable marks a state down for requiring participating
schools to administer the state's assessment or a norm-referenced test, to replace admissions
with open enrollment and a lottery, to accept the scholarship as the full value of tuition, or to
hold accreditation. Ohio's 12th on Education Choice is therefore a composite of how many students
are eligible and participating *and* how little is asked of the schools they attend.

**Why it is catalogued.** It is the most-cited external ranking of Ohio's choice programs and the
only retrieved source that scores Ohio on choice and on spending in one instrument. Its point of
view is not incidental to the numbers — the scoring direction on spending is a policy position
expressed as a rank — and cataloguing it with that stated is the only way a corpus node can use
it without importing the position. The same treatment
[`fordham-base-cost-critique`](fordham-base-cost-critique.md) gets, for the same reason.

**Access constraints.** The highlights booklet is a freely available static PDF and was retrieved
and read. The interactive report card at `educationreportcard.heritage.org` returns **HTTP 403 to
anything that identifies itself as a program**, which is the same failure mode as one of the two
[blocked connectors](../../crates/connect/README.md) — so per-category *scores*, as against ranks,
and the **weighting of the four categories** are not established here. The booklet states the
categories and their variables but never states their weights. [open]

**Caveat — the per-pupil denominator is not this corpus's.** Heritage's nominal per-pupil spending
variable is "total current expenditures, capital expenditures, and interest on school debt per
pupil". The capital channel is invisible in every per-pupil operating figure this repository
computes, by declared scope. Heritage's spending figure and
[`metric/per-pupil-operating-expenditure`](../corpus/metric/per-pupil-operating-expenditure.yml)
therefore measure different things, and neither can be substituted for the other. Also note the
edition year: the 2023 card predates the first full year of Ohio's universal EdChoice expansion,
so Ohio's 12th on choice is a rank earned mostly before the expansion this corpus records.

## Used by

- [`program/edchoice-expansion`](../corpus/program/edchoice-expansion.yml)
