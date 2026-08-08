# LSC School Funding Complete Resource

**Source.** Ohio Legislative Service Commission, *School Funding Complete Resource*.
**Type.** Primary source for legislative practice — the legislature's own nonpartisan service
commission, describing the formula it drafts, for the members who vote on it.
**Location.** `lsc.ohio.gov/assets/organizations/legislative-service-commission/files/`,
`school-funding-resources-2008-school-funding-complete-resource.pdf` (November 2008, 95pp) and
`...-2011-...` (February 2011).

**What it contains.** A full walk of Ohio's school funding formula as it stood at publication:
the adequacy amount and its build-up, the charge-off rate and state share percentage, the
categoricals, local property tax and H.B. 920 reduction factors, phantom revenue by type, and
the interaction between the funding formula and tax policy.

**Why it matters here.** It is the only committed source that states the **charge-off rate and
the valuation base together**, which is the distinction that makes a charge-off series usable —
see [local share charge-off millage](../corpus/parameter/local-share-charge-off-millage.yml). It
is also where the charge-off supplement (gap aid) is quantified, and where the JVSD rate of 0.5
mills appears.

The 2008 edition covers the "building-blocks" formula; the 2011 edition covers the
Evidence-Based Model and states in its introduction that the previous model is described in the
2008 edition. Between them they span the two regimes immediately before the Bridge formula.

**Access constraints.** The host's TLS chain does not validate through standard fetching —
the same condition recorded for the LSC Members Brief in
[the DeRolph record](derolph-litigation-record.md). `curl` retrieves both files without
complaint, which is how they were read here. Text extraction needs a PDF text layer; both have
one.

**Caveat.** The 2011 file at the URL above is **eight pages** — front matter and table of
contents only — although its own contents list runs to page 69. The full 2008 edition does
retrieve completely. Anything cited from the 2011 edition must be checked against a complete
copy first; every figure this corpus draws from LSC comes from the 2008 file.

**Status.** *Retrieved and read, not extracted.* Nothing here is committed as a fixture. The
charge-off rate series is carried as constants in
[`crates/regime-diff`](../../crates/regime-diff/src/charge_off.rs) with its authority beside it,
because four values with citations are a parameter and not a data file.

## Used by

- [`parameter/local-share-charge-off-millage`](../corpus/parameter/local-share-charge-off-millage.yml)
- [`formula-component/charge-off-local-share`](../corpus/formula-component/charge-off-local-share.yml)
