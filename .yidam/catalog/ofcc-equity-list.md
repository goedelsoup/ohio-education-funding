---
used-by: []
---
# OFCC K-12 Equity List and CFAP Status Editions

**Source.** Ohio Facilities Construction Commission. The *K-12 School District Equity List (by
rank)*, one edition per fiscal year, and the rolling *Classroom Facilities Assistance Program
Status* edition, both recovered from the Internet Archive.
**Type.** Primary source. The Commission's own publication of the R.C. 3318.011 ranking the
department certifies to it, and the Commission's own account of where each district stands in
the program.
**Location.** `dam.assets.ohio.gov/image/upload/ofcc.ohio.gov/Portals/0/Documents/`. The Commission's
host refuses this repository's agent (see
[`the-capital-channel-has-no-second-publisher`](../decisions/the-capital-channel-has-no-second-publisher.yml)),
so every byte here comes from a Wayback capture fetched with `id_`.

**What it contains.** Each equity list ranks all 609 city, local and exempted village districts
from the lowest three-year average adjusted valuation per pupil to the highest. Each row gives
the rank, the district's name, its county and the valuation the rank was cut on. The status
edition explains the share rule in a paragraph, totals the districts by status, and prints every
district's current rank beside its status.

**The editions held, by capture.** Every capture was listed through the CDX index, downloaded with
`id_`, and compared on SHA-256.

| Edition | Capture | Bytes | Held |
|---|---|---|---|
| FY2025 list by rank, `Resources/Treasurers/K-12-Equity-List-FY25-_by-rank.pdf` | 2025-06-28 | 412,964 | `ofcc-equity-fy25` |
| FY2024 list by rank, `Resources/Ranking-Lists/2024/…(by rank).pdf` | 2024-07-01 and 2025-06-05, one digest | 446,576 | `ofcc-equity-fy24` |
| Status, rolling URL, as of 31 December 2025 (FY26 Q2) | 2026-10-17 | 611,492 | `ofcc-cfap-status` |
| Status, `…-Status-Dec-2023.pdf`, as of FY24 Q2 | 2024-06-13 | 546,518 | not committed |
| Status, `…-Status-Dec-2024.pdf`, as of 30 June 2024 (FY24 Q4) | 2024-09-08 | 545,303 | not committed |
| Status, `…-Status-Dec-2024.pdf`, as of 30 June 2024 (FY24 Q4) | 2025-06-28 | 812,761 | not committed |

**The Dec-2024 filename does not hold a December 2024 edition.** Both of its captures differ in
bytes, and both are the FY24 Q4 edition as of 30 June 2024. The second has no title text on its
first page. Only the rolling edition is committed, because it is the one that prints the FY2026
ranks.

**What is not archived.** No capture exists of the FY2026 list, or of anything under
`Ranking-Lists/2025/` or `Ranking-Lists/2026/`. The FY2026 list governs FY2027 approvals, and its
ranks survive only in the status edition. Older lists, FY2007 to FY2023, have captures on the
Commission's former host at `ofcc.ohio.gov/Portals/0/Documents/Resources/Ranking%20Lists/` and
are not read here. The Commission's meeting minutes have captures from 2013 to 2018. Those from
2024 on, which state each approved project's state and local dollars, are archived nowhere.

**Against the research folder.** The research copies of the FY2024 and FY2025 lists by rank, of
the Dec-2023 status edition, of the second Dec-2024 capture, and of the rolling edition each
match a Wayback capture byte for byte. The research folder also holds FY2026 lists by rank and
alphabetically, which no archive holds. With no publisher to fetch them from, they are not
committed. Its June-2023 and Sep-2023 status editions have no capture under these paths.

**How it is read.** `connect::fixtures::equity_ranks` reads each list's `pdftotext -layout` text
into `crates/dispersion/fixtures/ofcc-equity-ranks.csv`, one row per district per list year,
keyed on name and county. The lists carry no IRN. Lakota Local is in Sandusky and in Butler,
and Buckeye Local is in three counties. The reader refuses ranks that are not exactly `1..=n`.
**The FY2024 text layer prints ranks 445 to 448 twice**, mid-page and cell for cell. The reader
drops a row identical to one already read and refuses any other repeat. The status edition is
committed as text, `crates/dispersion/fixtures/ofcc-cfap-status.txt`, and is read by
`dispersion::equity_rank`.

**Caveats.**

- **The worked example is approximate.** The status edition says rank 307 "would have a
  state/local split of 50/50". 307 of 609 is 50.4%, which the statute's percentile puts at 51.
- **The list does not state a share.** The share is the percentile under R.C. 3318.01(K), capped
  at 95 by 3318.032(D). Before 30 September 2025, a district paid the greater of that and a
  net-indebtedness amount, and no edition states that amount per district.
- **The list year is the year before the approval.** A project the controlling board approves in
  FY2026 reads the FY2025 list.

## Feeds connector

`ofcc-equity-lists`, recorded in
[`ofcc-equity-lists-connector`](../decisions/ofcc-equity-lists-connector.yml).
