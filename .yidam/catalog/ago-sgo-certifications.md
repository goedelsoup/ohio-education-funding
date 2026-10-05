---
used-by: []
---
# Scholarship granting organization certifications — Ohio Attorney General

**Source.** Ohio Attorney General, Charitable Law Section, which certifies organizations under
R.C. 5747.73(C).
**Type.** Primary source — the certifying office's own list.
**Location.** `charitable.ohioago.gov/Scholarship-Granting-Organization-Certification/List`.

**What it contains.** One table for each year from 2021, each row an organization: its EIN, the
certification date, a "valid through" date, its legal name, any trade name, and an address.

**Why it is not digest-pinned.** The page is an ASP.NET form. Each table shows ten rows a page,
the later pages are reached by posting the form back with its `__VIEWSTATE`, and every response
carries a fresh view state and anti-forgery token. No two retrievals have the same bytes, so a
pinned digest would fail on every fetch. The list is held instead as
`crates/project/fixtures/sgo-certifications.tsv`, extracted by hand on **2026-10-05** by paging
every table through its postback. That date is the only day its counts are true of, and
`project::donation_credit::RETRIEVED` carries it.

**What the extract found.** 296 rows in six tables, 89 distinct EINs. Three organizations hold two
rows in one table, each a recertification within the year. Four rows sit in a table other than
their certification year. Three 2026 certification dates fall after the retrieval date. One
certification is valid for no days. Two certified organizations give addresses outside Ohio, one
in Pennsylvania and one in Virginia, and the statute does not require an Ohio organization. One
Cincinnati address is entered with the state "Alabama". All of these are kept as printed.

**The term is the list's, not the statute's.** R.C. 5747.73 sets no term for a certification.
Almost every row is valid for 364 or 365 days, which is the Attorney General's practice.
