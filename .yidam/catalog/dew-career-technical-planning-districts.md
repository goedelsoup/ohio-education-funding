# Ohio Career-Technical Planning Districts

**Source.** Ohio Department of Education and Workforce. `Ohio-CTPDs.pdf`, a single-page map with
a table beneath it, about 722 KB.
**Type.** Primary source — the department's own designation of each planning district and its
lead, not a report about them.
**Location.** `education.ohio.gov`, Topics → Finance and Funding → State Funding for Schools →
Career-Technical Funding.

**What it contains.** One row per career-technical planning district: county, CTPD number, the
name of the lead district, the lead's **IRN**, its city, and which of four kinds the planning
district is — comprehensive district, JVSD, compact/contract, or correctional institution. The
table carries **92** rows. Above it a map legend breaks the same population into 26 comprehensive,
15 compact/contract, 49 JVSD and 2 correctional, which sums to 92 against a printed total of 91;
the legend disagrees with itself and the table is the countable artefact.

**Why it was retrieved.** R.C. 3317.014(E)(1)(a) pays career awareness and exploration funds to
*the lead district of each career technical planning district*, on the summed enrolled ADM of that
district's members. That is the only payment in the corpus whose recipient is a consortium rather
than a district, and
[`parameter/career-technical-category-multiples`](../corpus/parameter/career-technical-category-multiples.yml)
had recorded the recipient as unknown and its availability as unprobed. This table answers half of
it: the leads are published, they are named, and they carry the identifier the panel is keyed on.

**What it does not contain, and what that costs.** Membership. The table names the lead of each
planning district and not the districts, community schools and STEM schools affiliated with it, so
it cannot produce the ADM the payment is computed on. That matters less than it looks, because the
department states that "every school district belongs to or serves as a Career Technical Planning
District" — a population with no non-members sums to the whole, so a statewide total needs no
roster at all. Only attribution to a recipient does.

**Access constraints.** Freely available, no authentication. `pdftotext -layout` recovers the table
cleanly; the map above it becomes a scatter of three-digit labels that should be discarded rather
than parsed. Two ligature artefacts survive extraction — `ﬁ` and `ﬀ` in names like Springfield,
Mayfield, Canfield, Fairfield and Jefferson — so a name match against this file needs normalising
first, while the IRN column does not.

**Caveats.**

- **The two portals that published membership are gone.** `webapp2.ode.state.oh.us/ctpd_region/`,
  the map whose per-CTPD pages listed each planning district's associate and member schools, no
  longer resolves in DNS. The report card site's CTPD pages
  (`reportcard.education.ohio.gov/ctpd/...`) return one identical 11,166-byte application shell
  for **every** path, `/robots.txt` included, so a 200 from it is not evidence that a route exists
  and nothing can be concluded from it without a browser. A membership roster is a live question,
  not a settled absence. [open]
- **This is a designation list, not a payment file.** It says who leads each planning district. It
  does not say what any of them was paid, and the department's career awareness payments are not
  published per recipient anywhere this corpus has found. The money is visible only in aggregate,
  as an earmark inside GRF line item 200545.
- **It is undated.** The file carries no revision date and no fiscal year, so it states the
  designations as of retrieval and cannot be used to date a change in them. A CTPD that
  reorganised would be invisible against an earlier reading.
- **The CTPD number is not an IRN and mostly not a district.** Numbers run `200001`-`200121` with
  two correctional entries at `200600` and `200602` and one outlier, `021357`, that does not share
  the prefix. The lead's IRN is a separate column and is the one that joins to anything here.

## Used by

- [`parameter/career-technical-category-multiples`](../corpus/parameter/career-technical-category-multiples.yml)

## Feeds connector

None. Nothing extracts this file; it was read to establish that the lead-district roster is
published and that membership is not. An extractor would be worth scoping only alongside a
membership source, since the leads alone cannot compute the payment.
