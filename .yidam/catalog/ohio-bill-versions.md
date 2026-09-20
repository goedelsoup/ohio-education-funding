# Ohio bill versions — what was proposed, not what was enacted

**Source.** Ohio General Assembly, via the Legislative Information Systems service that backs
`legislature.ohio.gov`.
**Type.** Primary source, of a bill version — the document as introduced or as it stood at a
named stage. **Not** authority for what the law is.
**Location.** `search-prod.lis.state.oh.us/api/v2/general_assembly_{GA}/legislation/{bill}/{version}/pdf/`.

**What it contains.** The whole bill, in amending form: existing sections printed with their
changes, new sections printed entire, and a title enumerating every section amended, enacted and
repealed. Line numbers run down the right margin, which survive `pdftotext -layout` as bare
integers inside sentences and have to be read around.

## Why a bill that never became law is worth committing

Because it answers a question the enacted text cannot. An act carries its provisions without their
history: R.C. 3317.0217(C)(1) in force today adjusts its denominator for one of the ten channels
R.C. 3317.03(A)(2) lists, and nothing in the Revised Code says whether that was a drafting choice
or a budget-process residue.

H.B. 1 of the 134th settles it, because it enacts R.C. 3317.0217 and amends R.C. 3317.03 in the
same document. The denominator already reads exactly as it does now, and the channel list it cites
already has its ten entries. Same author, same bill, same sitting.

## What it must not be read as

**A bill version is not law, and where a bill and the act that followed differ, the act governs.**
H.B. 1 of the 134th passed the House and died in the Senate; H.B. 110 enacted the formula four
months later. Any figure, rate or threshold quoted from a bill version is a *proposal*, and this
corpus has already been wrong once in exactly that shape — six claims cited LSC's redbook, which
analyses a bill as introduced, as though it were the appropriation. See
[`the-four-genres-of-a-description`](../decisions/the-four-genres-of-a-description.yml) and
`ledger::budget_analysis::Edition`, which exists so that a caller has to name which document it is
reading.

So the safe use is **structural** — which provisions were written together, in what order, citing
what — and the unsafe use is **quantitative**. Nothing in this repository takes a number from here.

## What is retrievable

The version code is the path segment: `00_IN` as introduced, and later stages count up. The
legislature's own document index for a bill is client-rendered, so scraping the HTML page returns
a shell with no links; the API above is the route.

**There is no LSC analysis of H.B. 1 of the 134th.** LSC writes greenbooks for enacted budget acts
and redbooks for executive proposals, and this bill is neither. The asset names that would hold one
return 404 while known-good LSC URLs return 200 from the same host, so the absence is real rather
than a naming guess. The bill text is the whole of what exists.
