# Ohio bills before enactment — the text as introduced

**Source.** Ohio General Assembly, via the Legislative Information Systems service that backs
`legislature.ohio.gov`.
**Type.** Primary source — a bill as it was introduced, which is what a sponsor put in and not
what anyone voted on.
**Location.** `search-prod.lis.state.oh.us/api/v2/general_assembly_{GA}/legislation/{bill}/00_IN/html/`.

**What it contains.** The whole bill: sponsors, the long title enumerating every Revised Code
section it would amend, and the amended text of each section. H.B. 643 of the 136th is one
section and eleven kilobytes; H.B. 96 of the same General Assembly is the operating budget and
amends more than two thousand.

**Why this is a third artefact and not one of the two already here.**
[`ohio-revised-code`](ohio-revised-code.md) serves the Revised Code **as it stands today**, and
[`ohio-session-laws`](ohio-session-laws.md) serves acts **as they were passed**. A pending bill
is neither: it is not in the code and it has not been enacted, and it may never be either. The
same distinction that put the session laws in their own record rather than folding them into
`ohio-laws` — recorded at
[`the-acts-themselves`](../decisions/the-acts-themselves.yml) — applies again one document
earlier.

## The version index answers the question the enrolled acts left open

`ohio-session-laws` records that an act's enrolled version code is **positional** — `06_EN` for
H.B. 215, `05_EN` for H.B. 650 and H.B. 770, `08_EN` for H.B. 282 — so it differs per bill and a
guess returns a nine-byte 404 that reads exactly like "not served". The fix it named was to read
the index at `.../legislation/{bill}/` first.

That index turns out to give the whole sequence, and it makes the *as-introduced* case simpler
than the enrolled one rather than harder. Introduction is always the first version, so the code
is always `00_IN`. For H.B. 96 of the 136th the index returns eight entries:

    00_IN   As Introduced
    01_RH   As Reported by the House Finance Committee
    02_PH   As Passed by the House
    03_PSC  As Pending in the Senate Finance Committee
    04_RS   As Reported by the Senate Finance Committee
    05_PS   As Passed by the Senate
    06_CR   As Reported by the Committee of Conference
    07_EN   As Enrolled

So `00_IN` needs no lookup and every other stage does. The index is also the only reliable way to
tell a pending bill from an enacted one: the listing endpoint reports a bill's **first** version
rather than its current one, so H.B. 186 of the 136th appears there as `As Introduced` and is in
fact enrolled with an effective date of 20 March 2026. Reading a bill's status off the listing
would record several enacted acts as pending.

## What this can be trusted for, and what it cannot

**It can be trusted for what a sponsor proposed.** The text is the document, served as HTML with
the amended sections inline.

**It cannot be trusted as a description of what will happen**, and the reason is stronger than the
ordinary caution about proposals. A bill's text **moves under its own URL** — `00_IN` is stable,
but a reader who cites "H.B. 643" without a version is citing whichever stage is current when
somebody follows the link. Anything read from here is pinned by digest for that reason, and a
[`draft-legislation`](../corpus/draft-legislation/) node states the version it was written from
in a field of its own.

**It carries no fiscal analysis.** The Legislative Service Commission's bill analyses and fiscal
notes are separate documents that [`lsc-budget`](lsc-dew-redbook.md) does not retrieve — the gap
[`hb-583-2022`](../corpus/legislation/hb-583-2022.yml) already records for an enacted act, and it
applies with more force to a pending one, where the LSC note is often the only estimate anybody
has. [open]

## Why no parser

There is deliberately none, and this connector stays at `retrievable` rather than being wired.
Turning a bill into provisions is reading, not extraction: deciding that a section amending
R.C. 3310.032 changes eligibility rather than an award amount, and that no lever in this
repository expresses it, is a judgement about the funding system. A parser that produced a
provision list from section headings would produce something that looked authoritative and was
not.

What the retrieval is for is the text itself, pinned, so that a draft node can be checked against
the document it claims to describe.
