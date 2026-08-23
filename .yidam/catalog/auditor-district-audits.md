# Auditor of State — district and service centre audit reports

**Source.** Auditor of State of Ohio. Annual financial audits of school districts and educational
service centres.
**Type.** Primary source, of an unusual kind — a **state officer reciting a local body's act**.
Every other primary source this repository holds is the acting body's own document.
**Location.** `ohioauditor.gov/auditsearch/Reports/<year>/<Entity>_<yy>_<County>.pdf`.

**What is held.** Five reports, chosen for one paragraph each. They are not read as financial
statements; the fixture is `crates/dispersion/fixtures/territory-transfers.tsv`, and the column
that matters is `recital` — the sentence in which the auditor states that a resolution transferred
one district's territory to another.

| report | recites |
|---|---|
| Bettsville Local FY2014 (final) | its own dissolution into Old Fort Local |
| Old Fort Local FY2015 | the same transfer, from the receiving side |
| Berkshire Local FY2016 | Ledgemont Local merged into it, "due to financial difficulties" |
| West Geauga Local FY2020 | Newbury Local transferred to it, **under O.R.C. 3311.22** |
| Geauga County ESC FY2019 (final) | its own dissolution into the ESC of the Western Reserve |

## Why a recital and not the order

R.C. 3311.22 puts the order in an **educational service center governing board's minute book**.
Those are not published: Geauga County ESC's sit in a vendor system that refuses every
non-browser client, and North Central Ohio ESC's public archive begins in January 2022 — eight
years after the Bettsville resolution. Reaching the instruments is a records request under
R.C. 149.43, not a fetch.

The auditor's report is what is published, and it quotes the resolution by date and issuing body.
That is one hop from a corpus node. It is also all there is.

## The trap: the proposing body's own audit says nothing

**Geauga County ESC's final report spans both Newbury resolutions and never says "Newbury".** Its
FY2015 report spans the Ledgemont resolution and never says "Ledgemont". North Central Ohio ESC's
FY2014 report names Bettsville only as a member of an insurance pool.

The act surfaces in the **receiving district's** audit and nowhere in the **proposing body's**.
Anyone who looks for a transfer where the order was made finds nothing and concludes, wrongly,
that nothing happened.

## One transfer is recited twice, and that is the check

Bettsville's own final audit and Old Fort's the following year carry the same resolution, the same
date and the same effect — two audited entities, two reports, one transaction. The sentences differ
only in where they put the definite article. Nothing else here has that kind of corroboration, and
it is why the recital is usable as evidence rather than as a lead.

## Retrieval

The reports have stable direct URLs and serve to this project's user agent without complaint. The
search interface in front of them does not, and nothing here needs it:

- **`ohioauditor.gov` returns HTTP 200 with a 34,330-byte HTML body for any nonexistent path**
  under `/auditsearch/Reports/`. That byte count is a 404. This is the third distinct soft-404
  signature this repository has had to fingerprint, after LSC's 10,835-byte body and the State
  Board's 3,548-byte one.
- **AuditSearch itself is only machine-drivable with session state** — `__VIEWSTATE`,
  `__VIEWSTATEGENERATOR` and `__EVENTVALIDATION` carried with the six `ddl*` fields at their
  literal "All …" defaults, then a GET of `results.aspx` on the same cookie jar. Empty `ddl*`
  values return 500 and following the 302 with `-L` returns 411. Recorded so nobody works it out
  twice; the direct paths make it unnecessary.

## What this does not settle

**Which section Bettsville and Ledgemont ran under.** Only West Geauga's report names one. Berkshire's
cites R.C. 3311.241 for cancellation of solvency-fund debt on dissolution, which is a neighbouring
provision and may or may not indicate a different route. The `section` column is empty for both
rather than filled from the statute's shape.

**The instruments.** Three resolutions, in two minute books, neither published.

## Used by

- [`crates/dispersion/src/lea_directory.rs`](../../crates/dispersion/src/lea_directory.rs) —
  `transfers()` and `explained()`, beside the directory whose 689 departures they explain five of.
- [`the-order-was-never-the-states`](../decisions/the-order-was-never-the-states.yml), which
  established that the State Board never held these and pointed here.
