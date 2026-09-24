# Career-Technical Planning District Membership

**Source.** Ohio Department of Education and Workforce, Ohio School Report Cards data API.
`reportcarddataapi.education.ohio.gov/api/GetByIrn/{irn}/{schYear}`, plus the report card's own
organisation index `live-search.json`.
**Type.** Primary source — the department's own roster of each planning district's members.
**Location.** An **endpoint**, not a file. This is the only source in the catalog that is.

**Why it was retrieved.** R.C. 3317.014(E)(1)(a) pays the lead district of each career-technical
planning district $3 for every pupil in the summed enrolled ADM of that planning district's
members. [`dew-career-technical-planning-districts`](dew-career-technical-planning-districts.md)
gives the **92 leads** and records membership as a live question — "a membership roster is a live
question, not a settled absence. [open]". This is the roster.

**What it contains.** For each planning district: its name, county, address, superintendent, and
`detail.enrollmentDist` — **one entry per member, with that member's career-technical
enrolment**. For each *district*, the same endpoint returns `ctpd` and `ctpdIrn`, which state the
planning district it belongs to directly.

A 954-row extract for FY2025 is committed at
[`crates/dispersion/fixtures/ctpd-membership.csv`](../../crates/dispersion/fixtures/ctpd-membership.csv),
built by [`connect::fixtures::ctpd`](../../crates/connect/src/fixtures/ctpd.rs).

## Why an API, when this catalog is otherwise files

Because it is the only route, and that is checked rather than assumed. The report card's static
download catalogue publishes a `Career Technical Planning District` category of **twenty-four
files back to 2013** — `CTPD-Ratings_2025.xlsx`, `CTPD-Grades_2018`, and so on. Every one of them
is ratings or grades. **Membership appears in none.** The two portals that did publish it are
gone: `webapp2.ode.state.oh.us/ctpd_region/` no longer resolves in DNS, and the report card's own
CTPD pages are an Angular shell returning one identical 11,166-byte document for every path,
`/robots.txt` included.

**It is nonetheless pinnable by URL, which is what makes it a source here rather than a
procedure.** The endpoint's Azure Functions key — shipped to every visitor in the site's
`main.<hash>.js` — is accepted as a **`?code=` query parameter** as well as a header, so each
roster is a plain URL of the same shape as the report-card downloads already in the registry.

**The key itself is not committed.** The registry's URLs carry `{REPORT_CARD_KEY}`, and
`edfund-connect fetch` fills it from `EDFUND_REPORT_CARD_KEY`. Read it from the
`x-functions-key` beside `GetByIrn` in the site's current `main.<hash>.js` — the bundle's name
changes when the site is rebuilt, so find it from the page rather than guessing the filename.
The committed fixture needs none of this; only a refresh does.

**And the bytes are stable.** Each document carries Cosmos metadata (`_ts`, `_etag`, `_rid`) and
`_ts` is the last *write*, not the read: two fetches are byte-identical. So these pin by SHA-256
the way a published file does, and a changed digest means the department rewrote the record.

## The 92, the 90, and the two

The CTPD table has **92** rows and its map legend "disagrees with itself" — the caveat
[`dew-career-technical-planning-districts`](dew-career-technical-planning-districts.md) records
as unresolved. It resolves here. **Ninety** planning districts are rated and answer. The two that
are not are the correctional institutions `200600` and `200602`: they return **HTTP 204 No
Content** and appear in no organisation index, because nobody rates them. 90 + 2 = 92, and the
204 is a structural answer rather than a transient one.

## Caveats

- **`enrlCtpd` is career-technical enrolment, not enrolled ADM.** The statute pays on enrolled
  ADM and this source does not publish it. Every share computed from this fixture describes who
  does career-technical education through a planning district, **not** the base the payment
  multiplies, and nothing built on it returns a dollar.

- **A roster entry names its member and gives no identifier.** `enrollmentDist` entries carry
  `distName` and `enrlCtpd` and nothing else. Resolution is against the report card's own
  organisation index — 4,018 organisations with IRN, kind and name — rather than a match against
  this repository's panel.

- **Nineteen names are shared by forty-seven districts, and no roster can tell them apart.**
  Normalised for the two styles the publisher writes, 607 district names collapse to **579
  keys**: three `Perry Local`s, three `Buckeye Local`s, three `Madison Local`s. The index
  disambiguates with a ` - County` tail the roster does not have. Those forty-seven are therefore
  retrieved individually, because **a district's own record carries `ctpdIrn`**. No two
  same-named districts share a planning district — checked, not assumed — so the placement is
  unique, and the builder refuses a shared name it cannot place rather than picking a candidate.

- **347 of the 954 members are not districts.** They are community schools under R.C. Chapter
  3314 and STEM schools under Chapter 3326 — the population the 609-district panel does not
  carry. They are 36.4% of the membership and **4.75%** of career-technical enrolment, and 277 of
  them carry none at all.

- **The organisation index is published per school year under the current year's prefix**
  (`2026appsettings/`). A prior year's prefix is not guaranteed to survive, so the index is a
  current-roster artefact and a re-retrieval years hence may not reproduce this one.

- **The key and the bundle filename both move when the site is rebuilt.** If the department
  rotates the key, every one of these sources returns 401 at once — a loud failure rather than a
  quiet one, and the repair is to re-read it from the current bundle. No amount of care here
  prevents the rotation; committing the key would only have made the failure later and more
  confusing.

- **FY2026 answers on the same endpoint and is not committed.** FY2025 is taken because it is the
  year whose membership was cross-checked against the other side of the same API: every district's
  own record carries a `ctpdIrn`, and all 607 agreed with the roster-derived placement.

## Used by

- [`parameter/career-technical-category-multiples`](../corpus/parameter/career-technical-category-multiples.yml)
