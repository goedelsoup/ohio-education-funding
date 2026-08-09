# Ohio Revised Code — the sections this corpus cites

**Source.** Ohio General Assembly, via the Legislative Service Commission's `codes.ohio.gov`.
**Type.** Primary source — the law itself, published by the body that writes it.
**Location.** `codes.ohio.gov/ohio-revised-code/section-<number>`, one page per section.

**What it contains.** The current text of each section, its **effective date**, and the **act that
last amended it**. Fourteen sections are retrieved: the Chapter 3317 foundation program sections
the formula components rest on, R.C. 319.301 for H.B. 920 and the twenty-mill floor, and
R.C. 5705.391 for the five-year forecast.

**The recorded blocker was a statement about a convenience, not about the data.** This connector
sat `Declared` since genesis on "codes.ohio.gov serves HTML with no bulk export; section history
is rendered rather than published as data." The first clause is true and was read as though it
meant the text was unreachable. **It is server-rendered**: the operative text is in the response
body of a plain `curl`. The second clause remains true, and the corpus takes the current text and
its effective date rather than attempting a version series.

**What it settled that nothing else could.** Every weight in the formula had been transcribed from
the department's spreadsheet and confirmed against the department's own published amounts — which
is a check that the department is self-consistent, not that it is following the law. Against
statute, **all fourteen multiples match to the last digit**: six special education, three English
learner, five career-technical. See
[`crates/project/tests/the_statute_behind_the_weights.rs`](../../crates/project/tests/the_statute_behind_the_weights.rs).

It also answered four questions the corpus had recorded as open, and corrected one citation:

- the **clinical categories** behind the special education weights, which R.C. 3317.013 names;
- what the **English learner taper** actually tracks — time enrolled in United States schools and
  attainment of a proficient score, per R.C. 3317.016;
- the **career-technical programme categories**, which R.C. 3317.014 lists by name;
- where the **squaring** in the disadvantaged-pupil index comes from — R.C. 3317.02(I)(1)(a),
  which the DPIA node had recorded as "not located in statute here";
- and **R.C. 3317.029 does not exist.** The DPIA node cited it. The programme is R.C. 3317.022(A)(4).

**How it is read.** `connect::html` — a text extractor rather than an HTML parser, because nothing
here wants a tree. It drops `<script>` and `<style>` with their contents, treats block tags as line
breaks and inline tags as transparent, and resolves the entities the site emits. The inline
distinction matters more than it sounds: a cross-reference is marked up as
`section <a>3317.011</a> of the Revised Code`, and breaking on every tag makes that unsearchable.

**Vintage risk.** Sections carry an effective date and most of these read September 30, 2025 —
H.B. 96 of the 136th General Assembly, the current budget. A future budget rewrites them at the
same URLs. The digest manifest is what detects it, and the weight-verification test is what makes
a silent divergence between statute and the committed weights fail loudly.

**Caveat.** The fixture is the *current* text. Any claim in the corpus about an earlier regime —
the charge-off era above all — cannot be sourced from here, because the archive begins at 1 July
2014 and the mechanism was already gone. Those claims still rest on the opinions and on session
law.

## Used by

- [`crates/project/fixtures/revised-code.txt`](../../crates/project/fixtures/revised-code.txt)

## Feeds connector

[`ohio-laws`](../../crates/connect/sources/ohio-laws.md)
