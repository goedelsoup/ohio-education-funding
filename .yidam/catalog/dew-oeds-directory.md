---
used-by: []
---
# Ohio Educational Directory System: nonpublic school extract

**Source.** Ohio Department of Education and Workforce, **Ohio Educational Directory System
(OEDS)**, the public data extract.
**Type.** Primary source — the department's own register of organisations, generated on request.
**Location.** `https://oeds.education.ohio.gov/DataExtract/GetRequestOrgExtract`, as a POST of one
form field, `jsonData`, naming the organisation types wanted. Type 5 is Nonpublic School. A GET of
the same URL is refused with a 403 by the department's gateway, so the registry carries the form
(`connect::registry::REQUESTS`) rather than only a URL.

**What it contains.** One row per open organisation per school type, 24 columns: building IRN,
name, school type, grade span, status, designated county, web address, email, mailing address,
phone, principal, and a parent IRN and name. For nonpublic schools the parent is the diocese or
association — the Cleveland Catholic Diocese is parent to 114 — and **never the public school
district**. There is no district column. Every IRN is written as an Excel formula, `="057539"`, so
a spreadsheet keeps the leading zero, and the first line is `Generated on <timestamp>` ahead of
the header.

When first fetched it listed **787 open nonpublic schools in 903 rows**: a school offering
elementary and middle grades appears once per type, with the same name and address. The
extraction collapses them and fails if two rows for one IRN ever disagree.

**Why this is here.** R.C. 3317.024(E)(1) pays auxiliary services through the district a
chartered nonpublic school is located in, and no October enrolment file says which district that
is ([`dew-nonpublic-enrollment`](dew-nonpublic-enrollment.md)). This extract is the only public
record of where a nonpublic school is: an address against a building IRN, which the October files
share. The address is geocoded into the 2020 census blocks by the Census Bureau's batch geocoder,
and the [block assignment file](census-block-geography.md) gives the district each block lay in.
`crates/project/fixtures/nonpublic-school-district.csv` is the result. The decision record
`nonpublic-school-location` sets out why that is a keyed join with a stated reach, not the
geographic match `nonpublic-enrollment-connector` rejected.

**What it does not support.**

- **Anything before the day it was generated.** It is a snapshot: schools open that day, at that
  day's address. A school that has moved is placed where it is now, and a school that has closed
  is absent. Six buildings in the October 2025 count are already missing from it, and 23 of
  October 2023's.
- **A digest that means anything about the schools.** The `Generated on` line changes on every
  fetch, so the digest moves whether or not a school did.
- **The `Selected` field.** The request carries a list of columns wanted and the extract ignores
  it, returning all 24 whatever is asked for.
- **Closed organisations.** The directory's search API (`Api/searchOrg`) reaches them and is not
  read.
