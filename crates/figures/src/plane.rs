//! The third chart registry: [`PLANES`], a handful of named alternatives on two signed axes at
//! once, where the finding is that the axes disagree.
//!
//! Seven policies rather than six hundred subjects, each of which a reader quotes by name and by
//! both coordinates — so every [`Position`] names a figure for each of its two, and the module is
//! four items long because there is nothing to compute here that the figure registry has not
//! computed already.

use super::*;

/// One plane the wiki draws: a few named alternatives, each placed on two signed axes at once.
///
/// The third drawing, and a different shape again. A [`Series`] is one measure of several things;
/// a [`Scatter`] is one population under one predictor. A plane is *two measures of the same few
/// things*, drawn together because the finding is that the two measures disagree — a rule can
/// write less guarantee than the rule in force and still pay more total state support, and no
/// table that reports one column at a time can show it.
///
/// Both axes are **signed** and both carry a rule at zero, so the reading is by quadrant: a point
/// below and left is cheaper on both measures, a point above and left is the disagreement. There
/// is no fitted line, because seven policy alternatives are not a sample of anything.
///
/// What stands in for the endpoint rule is stricter than either of its siblings: every
/// [`Position`] names a [`Figure`] for **both** of its coordinates. A cloud may pin two numbers
/// out of twelve hundred because no single point is the claim; here every point is quotable by
/// name, so every coordinate is pinned.
pub struct Plane {
    /// `<crate-directory>/<what-it-is>`, in the one key namespace the other registries share.
    pub key: &'static str,
    /// The crate that owns the computation, as the corpus cites it.
    pub owner: &'static str,
    /// What the whole drawing is, in words; its title.
    pub label: &'static str,
    /// What one point is, singular and lower case — "anchor rule". Written into the caption.
    pub subject: &'static str,
    /// The computation, over the same inputs the figures use.
    pub compute: fn(&Inputs) -> Positions,
}

/// What the seven rules cost on both axes at once. See [`Plane`] for why this is its own shape.
///
/// #444's first drawing, and the one it said should exist even if nothing else here did. Three of
/// the seven sit in the upper-left quadrant — less guarantee written, more total state support
/// paid — and that disagreement is invisible in any table that reports one column at a time.
pub static PLANES: &[Plane] = &[Plane {
    key: "project/what-each-anchor-rule-costs-on-both-axes",
    owner: "crates/project",
    label: "What each anchor rule costs outside the enrollment cluster, on the guarantee it \
            writes and on the total state support it pays",
    subject: "anchor rule",
    compute: |i| {
        let coordinates: [(&'static str, &'static str); 7] = [
            (
                "project/a-ratchets-guarantee-written-outside-the-cluster",
                "project/a-ratchets-total-state-support-outside-the-cluster",
            ),
            (
                "project/a-caps-guarantee-cut-outside-the-cluster",
                "project/a-caps-total-state-support-cut-outside-the-cluster",
            ),
            (
                "project/the-gentle-caps-guarantee-cut-outside-the-cluster",
                "project/the-gentle-caps-total-state-support-gain-outside-the-cluster",
            ),
            (
                "project/the-rolling-counts-guarantee-cut-outside-the-cluster",
                "project/the-rolling-counts-total-state-support-gain-outside-the-cluster",
            ),
            (
                "project/the-mirror-inside-h-guarantee-cut-outside-the-cluster",
                "project/the-mirror-inside-h-total-state-support-gain-outside-the-cluster",
            ),
            (
                "project/the-mirror-beside-guarantee-written-outside-the-cluster",
                "project/the-mirror-beside-total-state-support-gain-outside-the-cluster",
            ),
            (
                "project/the-dated-phase-downs-guarantee-written-outside-the-cluster",
                "project/the-dated-phase-downs-total-state-support-outside-the-cluster",
            ),
        ];
        let points = i
            .reach
            .rules()
            .into_iter()
            .zip(coordinates)
            .map(|((label, frame), (x_figure, y_figure))| {
                let totalled = i.reach.aggregate(frame);
                Position {
                    label: label.to_string(),
                    x: totalled.guarantee_delta,
                    y: totalled.delta,
                    x_figure,
                    y_figure,
                }
            })
            .collect();
        Positions::framed(
            "Guarantee written, against the enacted anchor",
            "Total state support, against the enacted anchor",
            Unit::Dollars,
            points,
        )
    },
}];

/// A plane and the positions it came out with on this run.
pub struct ComputedPlane {
    /// The registry entry.
    pub plane: &'static Plane,
    /// What [`Plane::compute`] returned.
    pub positions: Positions,
}

/// Run every plane in [`PLANES`], in registry order.
#[must_use]
pub fn compute_all_planes() -> Vec<ComputedPlane> {
    let inputs = Inputs::shared();
    PLANES
        .iter()
        .map(|plane| ComputedPlane {
            plane,
            positions: (plane.compute)(inputs),
        })
        .collect()
}
