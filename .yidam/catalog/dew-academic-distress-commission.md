---
used-by:
  - ../corpus/education-agency/cleveland-municipal.yml
  - ../corpus/intervention/academic-distress-commission.yml
  - ../corpus/legislation/hb-33-2023.yml
  - ../corpus/program/edchoice-scholarship.yml
---
# Academic Distress Commission — the department's commission page

**Source.** Ohio Department of Education and Workforce, Office of School and District Improvement,
*Academic Distress Commission*.
**Type.** Primary source — the administering office's own record of which districts are subject to
a commission and what they have filed.
**Location.**
`education.ohio.gov/Topics/School-and-District-Improvement/Identification-and-Requirements/Academic-Distress-Commission`.
Last modified 10 February 2026 at the time of reading.

**Status.** *Read, not retrieved.* No connector fetches it and no fixture is built from it. The
`ohio-laws` HTML reader keys on landmarks that only `codes.ohio.gov` prints, so this page needs a
reader of its own or a different approach. What is taken from it is a small number of dated facts,
transcribed into
[`intervention/academic-distress-commission`](../corpus/intervention/academic-distress-commission.yml).

**What it contains.** The trigger, stated as the department states it — "Districts are subject to
an academic distress commission after receiving an overall rating of less than two stars on the
Ohio School Report Cards for three consecutive years" — and then, per district, the approval
letters, approved academic improvement plans, modification proposals and annual reports.

**The three districts.** East Cleveland City Schools, Lorain City School District, and Youngstown
City Schools. These are the districts that were under commission oversight when the 2021 change in
state law required an academic improvement plan, and revised plans were approved for all three on
3 December 2021.

- **East Cleveland** carries a *Notice of Release from Academic Distress Commission*, and it has
  now been opened — see below.
- **Lorain**'s commission was dissolved by H.B. 33, signed 4 July 2023.
- **Youngstown** remains, with annual reports through 2024-2025 and an approved extension of its
  plan, both now opened — see below. By January 2026 the department was describing it as **the
  sole district in Ohio under academic distress**, and said removal "would have to be the result
  of state legislation".

## The Youngstown extension, opened

`YCS-Approval-of-Extension-1_26.pdf` at the same `getattachment/` path, two pages, read 2 October
2026, SHA-256 `70b89736…a447c`. It is a scanned image with no text layer, so it was read by eye,
not extracted. A letter from the director, **dated 22 January 2026**, to the superintendent and
board president. It says:

- the district asked for release under R.C. 3302.103, and the department found it "did not meet
  the majority" of the overall benchmarks in its three-year plan, so release is declined;
- on the superintendent's emailed request of 14 January 2026, the director has "approved an
  extension ... in accordance with R.C. 3302.103", and **"During the extension year (2025-2026
  school year)"** the district continues implementing the plan;
- the district's request to amend its benchmarks is refused, because R.C. 3302.103(D)(2)'s
  revision window ran from 1 July 2022 to 30 June 2025.

So this is the **first** of the two extensions division (F)(1)(a) allows. The approval post-dates
half the year it covers. It does not mention a second extension or say what follows the 2025-26
year.

`Youngstown-City-School-District-AIP-Annual-Report-2025.pdf`, 33 pages, read the same day,
SHA-256 `e9f3af32…2689c`. The district's own report: **6 of 24 benchmarks met in 2024-25**, after
9 in 2023-24 and 16 in 2022-23. It notes that it would have met 15 had the targets not risen in
the final year.

**The page itself was still "Last Modified 2/10/2026" on 2 October 2026.** It lists no second
extension application or approval. Whether Youngstown's plan route ended on 30 June 2026 or runs
into a second extension year is not on it. [open]

## The release notice, opened

`East-Cleveland-message-12_25.pdf` at
`getattachment/Topics/School-and-District-Improvement/Identification-and-Requirements/Academic-Distress-Commission/`,
one page, read 15 September 2026. It is a letter from the director of education and workforce to
Dr. Henry Pettiegrew II, superintendent of East Cleveland City School District, and it says the
district "has been released from Academic Distress Commission oversight", having met 16 of 20
benchmarks under its Revitalization Plan — a Performance Index up more than 12 points, a 4-star
Progress rating, an **overall rating of 3 stars**.

**It carries no date on its face.** That is the answer to the `[open]` mark this entry used to hold
for the release date, and it is an unfillable-as-asked answer rather than a retrieved value: the
letter has no date line. What dates it is everything around it —

- the department's own filename, `message-12_25`;
- the letter's closing promise to "visit the district in January";
- the PDF's creation timestamp, 9 February 2026, which dates the *web publication* and not the
  letter, the page having been last modified 10 February 2026;
- press reporting of 2 January 2026 that the letter was "sent in late December" 2025.

So: **late December 2025**, bounded by the letter's own contents rather than stated in it. Recorded
at that precision and no finer. The district had been under state control since 2018.

**Release is dissolution, not a lesser status, and the section is R.C. 3302.103.** The letter's
"16 of 20 benchmarks" is the test in R.C. 3302.103(F)(2), the section H.B. 110 enacted for the
three districts on this page: a district that meets "at least a majority of the academic
improvement benchmarks" in its plan has its commission "dissolved". While the plan runs,
3302.103(E)(1) says the district "shall not be subject to section 3302.10", so division (N) was
not available to East Cleveland from July 2022. This entry used to read the release as
3302.10(N)(1). It was not, and a first three-star year could not have completed (N)(1) anyway.
Either way the commission no longer exists, so this page's "released" is still R.C.
3310.03(E)(2)'s "ceases to exist".

**Three commissions, three different ends, and none of them under division (N).** East Cleveland
met its plan benchmarks. Lorain exited by legislation. Youngstown is in an approved extension of
its plan, and a district on that route leaves early only by statute, which is what the department
told reporters. It is not held by the two-to-three star band
[`intervention/academic-distress-commission`](../corpus/intervention/academic-distress-commission.yml)
describes, because R.C. 3302.10 does not apply to it while the plan runs.

**Cleveland is not on this list**, and the omission is informative rather than an oversight.
The *former* R.C. 3302.10 is the pre-2015 regime that the current section refers to when it speaks
of a commission "still in existence on October 15, 2015". R.C. 3302.103(A) lists the two
former-section commissions that survived into the current one, established in 2010 and 2013, and
those are Youngstown's and Lorain's. So if Cleveland was ever under a commission, it had left by then. The
corpus recorded Cleveland as "subject to state academic distress intervention for part of the
period" without distinguishing the two regimes.

**Caveats.**

- **It is a current-status page, not a history.** It lists the districts that were under
  commissions from 2021 and what has happened to them since. Anything before the 2015 rewrite —
  including Cleveland's — is not here.
- **No dates for establishment.** The page gives the 2021 plan requirement, the 2023 Lorain
  dissolution and East Cleveland's release, and does not say when any commission was established.
  R.C. 3302.103(A) supplies the years, 2010, 2013 and 2018, without naming districts. East
  Cleveland's is the 2018 one, from the reporting around its release. Which of Youngstown and
  Lorain is 2010 is not on this page or in any held source. [open]
- **The linked documents are not held.** Approval letters, improvement plans and annual reports are
  separate PDFs. Three have been read: East Cleveland's release notice, Youngstown's extension
  approval and its 2024-25 annual report. None is committed. They would carry the benchmarks each district was held to, which is the only place
  the corpus could learn what a commission actually required. [open]
- **A page, not a dataset.** Any figure taken from it is a transcription, and there is no digest
  behind it. Treat claims sourced here as weaker than claims sourced from a pinned fixture.
