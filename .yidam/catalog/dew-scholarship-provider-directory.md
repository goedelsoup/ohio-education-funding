---
used-by: []
---
# Ohio Scholarship Providers Interactive Directory — the list the statute requires, unread

**Source.** Ohio Department of Education and Workforce, *Ohio Scholarship Providers Interactive
Directory*.
**Type.** Primary source, published by the program administrator under the duty H.B. 96 wrote into
R.C. 3310.41(O): "The department shall maintain a list of each registered private provider and the
location of that provider on its publicly accessible web site."
**Location.** `https://education.ohio.gov/Topics/Other-Resources/Scholarships/Additional-Scholarship-Resources/Ohio-Scholarship-Providers-Interactive-Directory`,
which answers with a 301 to
`https://reports.education.ohio.gov/report/nonpublic-data-scholarship-provider-directory`. The
Additional Scholarship Resources page links the second address directly. **Not fetched and not
pinned.**

**Where it is cited.** The 2025 Scholarship Annual Report says, in each program's Providers section,
that "a list of providers and their contact information is available on the Department's website".
The only provider link in that PDF's annotations is this directory. The FY24 edition of the
[JPSN annual report](dew-jpsn-annual-report.md) links the same address. Neither the Autism page nor
its Provider Information page attaches a list of its own, and the Autism page tells nonpublic school
providers to keep their OEDS record current. Read 2026-10-05.

**Why it is not read.** The reports portal serves a byte-identical Angular shell at every path, so
neither its 200 nor a 404 carries any information. Its reports are Power BI embeds behind an
entitlement the portal's anonymous token does not carry, and that token's request returns `{}`.
Probed 2026-09-15 and recorded on
[#10](https://github.com/goedelsoup/ohio-education-funding/issues/10). Nothing this project can
fetch shows the directory's columns.

**Whether it is keyed on IRN is not established.** A nonpublic school has an IRN through OEDS, which
is what the Autism page's instruction implies the directory reads. A registered private provider
that is not a school, such as a therapy practice, has no reason to hold one. A list covering both
would key one population on IRN and the other on something else. [open]

**Why it matters here.** It is the only per-provider record of the scholarship channel the
department publishes, and for the two special-needs programs the statute requires it to give each
provider's location. It would place the more than 2,100 approved providers the consolidated report
counts across all five programs. Read anonymously, it places none.

**What would reach it.** An OH|ID session on the portal, the route the per-district Jon Peterson
breakdown also waits on. Or a public-records request for the list, which the statute makes a
public document.
