# intervention — actions

## Queries

- **Fiscal exposure.** Given an agency, follow `imposed-on` to any intervention in force and read
  `fiscal_effect`. A funding figure for a district under an academic distress commission is an
  amount computed by the formula and spent by somebody the formula does not know about.
- **Trigger trace.** Follow `triggered-by` to the metric, then to whatever else consumes it.
  The Ohio School Report Card overall rating reaches both this class and
  `formula-component/fsfp-performance-supplement`, which is the double duty the accountability
  expansion exists to make visible.
- **Authority check.** Follow `authorized-by` and read the section. An intervention's powers are
  enumerated in statute and are the part most often summarised wrongly — "state takeover" is not a
  term the Revised Code uses.
- **What has this node been wrong about?** Read `revisions:`. Each entry carries the claim as
  it stood, what replaced it, and the test or source that settled it — so a withdrawn figure can be
  recognised rather than re-derived, and `found_by` gives the check to re-run.

## Writing a node

- **Name the powers from the section, not from reporting.** The list in R.C. 3302.10(D) runs to
  seventeen items and the fiscal one is (j). A node that says "the state takes over the district"
  has lost every check a reader could make.
- **Separate what the intervention does to authority from what it does to the amount.** An academic
  distress commission does not change a single term of the funding formula. Conflating the two
  would put a mechanism in the corpus that appears to move money and does not.
- **`fiscal_effect` may be empty and should be, where it is.** Most of the ESSA ladder is
  instructional. An empty field there is accurate; a strained one is not.
