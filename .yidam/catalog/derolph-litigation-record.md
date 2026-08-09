# DeRolph Litigation Record

**Source.** Supreme Court of Ohio opinion archive (`supremecourt.ohio.gov/rod/docs`) for the
opinions themselves; the Bricker Graydon *Chronology of the DeRolph v. Ohio School Funding
Litigation* (`bricker.com`) for the procedural sequence; the Legislative Service Commission
Members Brief *DeRolph v. State School Funding Case* for the legislative-facing summary.
**Type.** Primary source (opinions) with two secondary compilations.

**What it contains.** The full twelve-year record of *DeRolph v. State* — filing in Perry
County on 19 December 1991, the 1993 trial before Judge Linton Lewis Jr. (70+ witnesses, 500+
exhibits), the 1994 trial ruling, the 1995 reversal on appeal, four Supreme Court of Ohio
decisions, and the 2003 writ of prohibition that ended it.

The chronology is the reason this entry exists separately from the opinions. The sequence is
easy to get wrong from the opinions alone, and one step in particular is routinely
misdescribed: *DeRolph IV* did not simply reaffirm the earlier holdings — it **vacated**
*DeRolph III* and reinstated *DeRolph I* and *II* as controlling law. Any account that treats
the four decisions as cumulative is wrong about what the operative holding is.

**Access constraints.** The four opinions are freely available with WebCite citations, and are no
longer only available: all four are retrieved, digest-pinned in the manifest, and committed as a
text extract. The Bricker chronology is a law-firm publication — reliable on procedure, but a
secondary source, and claims drawn from it are tagged accordingly rather than as verified primary
text. The LSC Members Brief PDF is served from a host whose TLS chain does not validate through
standard fetching; retrieve it by other means or cite the opinions directly.

Trial-level rulings are a different matter and are **not** reachable from this source: a common
pleas decision is not in the supreme court's archive, which is why the 2025 EdChoice ruling stays
sourced to reporting.

**Caveat.** Counts of "how many DeRolph rulings" differ across sources — four Supreme Court
decisions plus a 1997 clarification and the 2003 prohibition action, which some summaries
count as five or six events. This corpus models the four numbered decisions and describes the
clarification and the prohibition inside the relevant nodes.

## Used by

- [`crates/regime-diff/fixtures/derolph-opinions.txt`](../../crates/regime-diff/fixtures/derolph-opinions.txt)
  — the committed extract of all four opinions, one record per case.
- [`litigation/derolph-i-1997`](../corpus/litigation/derolph-i-1997.yml)
- [`litigation/derolph-iv-2002`](../corpus/litigation/derolph-iv-2002.yml)
- [`education-agency/northern-local-perry`](../corpus/education-agency/northern-local-perry.yml)
- [`doctrine/adequacy`](../corpus/doctrine/adequacy.yml)
- [`parameter/local-share-charge-off-millage`](../corpus/parameter/local-share-charge-off-millage.yml)
  — *DeRolph I* ¶97 is the source for the charge-off rate progression and its Ohio Laws
  citations, which is the one place this corpus uses the opinions as a **statutory** record
  rather than a judicial one. Ohio Laws' online archive for R.C. 3317.022 starts in 2014, after
  the mechanism was retired, so the opinion is the accessible text.

## Feeds connector

[`ohio-courts`](../../crates/connect/sources/ohio-courts.md)
