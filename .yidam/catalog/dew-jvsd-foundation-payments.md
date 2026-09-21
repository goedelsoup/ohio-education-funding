# JVSD Foundation Payment Reports, FY2022–FY2027

**Source.** Ohio Department of Education and Workforce, Office of Budget and School Funding.
Six workbooks, `FY22_final-3_JVSD_Foundation_Payment.xlsx` through
`FY27_JVSD_SEP_Foundation_Payment.xlsx`.
**Type.** Primary source — what the department paid, per district, not a report about it.
**Location.** `education.ohio.gov`, Finance and Funding → School Payment Reports → State Funding
for Schools → **Career-Technical Funding** → Career Tech Payment Reports → Career Tech Payment
Reports in Excel Format. Openly served, no credentials, roughly 100 KB each.

**The path is the first thing worth recording, because it is why this looked unpublished.** The
department's other two formula populations sit under names that say what they are —
`Traditional-School-Districts/` and `Community-School-Funding/`, one directory apart. A joint
vocational district's reports are filed under *career-technical*, and **no path on the
department's site contains the words "Joint Vocational" at all**. Every guess at one 404s, and a
404 from a CMS that serves a styled error page at 200-sized length reads exactly like an absence.
The link that finds them is labelled `JVSD Payment Reports in Excel Format` on the
`School-Payment-Reports` index and points into the career-tech tree. Issue #371 recorded the
population as published-nowhere on the strength of three probes under the obvious name; it is
published, and has been since FY2014.

**And there is no JVSD calculator.** For traditional districts and for community and STEM schools
the department publishes a *simulator* — a working model, forward-looking. For these forty-nine
it publishes **payment reports**: what was paid, after the year closed. For FY2022 through FY2026
that is the better artefact, and for FY2027 it is the only one. The archive runs FY2014 to
FY2027; this corpus takes FY2022 forward, which is the Fair School Funding Plan era.

**What they contain.** Eight worksheets per year, 49 districts each — `Parameters`, `Base Cost`,
`Base Cost ADM`, `Summary SFPR`, `Detailed SFPR`, `Other Adjustments`, `CTPD` and (from FY2023)
`CTE Restricted Funding`. The `Base Cost` sheet is 76 columns of the five-component build-up
against staffing ratios: funded classroom teachers, career-readiness teachers, counsellors,
librarians, EMIS support, building leadership and operations, down to `[E] Aggregate Base Cost`
and `[F] District Base Cost Per-Pupil`. `Base Cost ADM` gives four years of enrolled ADM split
by grade band and career-technical FTE.

**The forty-nine agree with this corpus's own answer.** `dispersion::lea_directory::joint_vocational_districts()`
finds forty-nine JVSDs in the federal directory by two routes that never touch this file — the
agency name, and the property levy a JVSD charges and a service centre does not. The department's
IRNs match that set exactly, member for member. That is a third independent route, and it is the
department's own.

**Why it was retrieved: item 7 of H.B. 96.** The act rewrote the JVSD state share of base cost
into per-pupil form, and these files carry the whole method per district —
`[f]`/`[g]` the two valuations, `[h]` the lesser of them that is actually charged, `[i]` base cost
enrolled ADM, `[A1]` base cost per-pupil, `[A2]` local capacity per-pupil, `[A3]` the state share
percentage. The FY2025 file has `[f]`, `[g]`, `[h]` and `[A3]` and **none of the three per-pupil
columns**: they appear for the first time in FY2026, so the amendment is legible as a schema
change in the department's own workbook.

A six-year extract is committed at
[`crates/dispersion/fixtures/jvsd-foundation-payments.csv`](../../crates/dispersion/fixtures/jvsd-foundation-payments.csv),
built by [`connect::fixtures::jvsd`](../../crates/connect/src/fixtures/jvsd.rs), which refuses a
file whose charged valuation is not the lesser of the two published, whose local capacity is not
that valuation at half a mill, whose published state share percentage its own inputs do not
reproduce, or whose base cost dollars do not reproduce under the method that year's law
prescribes.

## Caveats

- **The bracket tags are not stable across the series, and two of the collisions are severe.**
  `[I]` is *Total State Support* in FY2024 and FY2025 and the *Base Funding Supplement* in FY2026
  and FY2027, with total state support moved to `[J]` — a $559m column and a $1.4m one under one
  letter. `[A1]` is *Aggregate Base Cost* in FY2024–25 and *Base Cost Per-Pupil* in FY2026–27,
  dollars against dollars-per-pupil. This is the same shift
  [`dew-sfpr-line-by-line`](dew-sfpr-line-by-line.md) records for the traditional district
  reports, in the same act, and the same rule applies: read the label text, never the tag.

- **The sheet names move once and the column spellings move twice.** Sheets are `JVSD_BaseCost` /
  `JVSD_DetailedSFPR` through FY2023 and `Base Cost` / `Detailed SFPR` from FY2024. Columns are
  CamelCase in FY2022 (`StateSharePercent`), plain labels in FY2023 (`State Share Percentage`),
  and tagged from FY2024 (`[A3] State Share Percentage`). No single spelling reads the series.

- **`Base Cost Enrolled ADM` names two different live columns.** `Detailed SFPR` `[i]` and
  `Base Cost` `[b]` are spelled identically and disagree on **five of the forty-nine** FY2026
  districts. The department's own local capacity is computed from the `Base Cost` sheet's.
  Reading the nearer one reproduces 44 of 49 and misses the rest by 1–5% — close enough to read
  as rounding, and it is not.

- **Per-pupil base cost is published twice at two precisions and the department pays on the
  rounded one.** `Base Cost` `[F]` carries `9612.6462793433` where `Detailed SFPR` `[A1]` carries
  `9612.65`, and the payment is the *rounded* figure times enrolled ADM times the share. Using
  the full-precision copy misses Apollo's FY2026 base cost payment by $3.42. The two agree to a
  cent, so a check written at cent tolerance passes while discarding the only thing that tells
  them apart.

- **FY2027 is a first payment, not a final, and its enrolment is last year's.** The September 2026
  report carries FY2026's `[a] Enrolled ADM` on **43 of the 49 districts**, and an ADM equal to
  base cost enrolled ADM on 39, because FY2027 enrolment does not exist in September 2026. Under
  prior law this would not have mattered — the state share multiplied a three-year average that
  was already known. Under item 7 it multiplies the current year, so FY2027's state share of base
  cost is provisional in a way no earlier year's is. The fixture marks it per row.

- **FY2022's `Base Cost` sheet is 72 columns where every later year is 76**, and what it lacks is
  `District Base Cost Per-Pupil`. Prior law never divided by a pupil, so the column had nothing
  to do until the amendment gave it a job. The fixture leaves it blank for that year rather than
  deriving it.

- **The reports are revised in place within a year.** FY2026 alone has twelve monthly editions
  plus `Final #1`; the ones taken here are the latest final of each closed year and the latest
  payment of FY2027. A digest change means a revision, not a new year.
