//! No lever reaches the browser without a checkpoint the browser has to reproduce.
//!
//! # The gap this closes
//!
//! `web/src/lib/policy.ts` is a second implementation of `project::policy::apply`, and the way it
//! stays honest is that `verify.ts` runs every checkpoint in the feed through it before the
//! scenario page is allowed to render. That is a strong guarantee about the levers the
//! checkpoints exercise and **no guarantee at all** about a lever none of them moves.
//!
//! A lever added to `Policy` with no checkpoint is the worst version of the failure the
//! checkpoint rule exists to prevent: the page renders, reports itself verified, and computes the
//! new lever wrongly or not at all. Nothing in the type system says otherwise — a `Policy` field
//! the TypeScript never reads is a field the TypeScript compiles fine without.
//!
//! Three of the eight levers were added at once when Stage 3 was worked, and each would have
//! passed every checkpoint that existed before it. The transportation floor is the sharpest case:
//! it moves no core foundation funding, so a browser that had not implemented it would reproduce
//! `realized_aid` on every existing checkpoint **exactly**.
//!
//! So the rule is stated here rather than remembered: every field of `Policy` is moved off
//! current law by at least one checkpoint.

use bundle::build;

/// **Every lever is moved by some checkpoint.**
///
/// Written against `PolicyShape`, which is what the feed carries and what the browser reads, so
/// it fails on a lever that reaches `Policy` and not the shape as well as on one that reaches
/// neither.
#[test]
fn every_lever_is_moved_by_at_least_one_checkpoint() {
    let shapes: Vec<_> = build::checkpoint_shapes();
    assert!(
        shapes.len() >= 8,
        "only {} checkpoints; the set has shrunk",
        shapes.len()
    );

    let law = &shapes[0];
    assert_eq!(law.guarantee, "as-enacted", "the first is current law");

    // Each entry: the lever's name, and whether some checkpoint moves it off current law.
    let moved: Vec<(&str, bool)> = vec![
        (
            "guarantee",
            shapes.iter().any(|s| s.guarantee != law.guarantee),
        ),
        (
            "guarantee_argument",
            shapes.iter().any(|s| {
                (s.guarantee == "rebase" || s.guarantee == "phase-out")
                    && s.guarantee_argument != law.guarantee_argument
            }),
        ),
        (
            "base_cost_scale",
            shapes
                .iter()
                .any(|s| s.base_cost_scale != law.base_cost_scale),
        ),
        (
            "minimum_state_share",
            shapes
                .iter()
                .any(|s| s.minimum_state_share != law.minimum_state_share),
        ),
        (
            "phase_in_general",
            shapes
                .iter()
                .any(|s| s.phase_in_general != law.phase_in_general),
        ),
        (
            "phase_in_dpia",
            shapes.iter().any(|s| s.phase_in_dpia != law.phase_in_dpia),
        ),
        (
            "dpia_directly_certified_weight",
            shapes
                .iter()
                .any(|s| s.dpia_directly_certified_weight != law.dpia_directly_certified_weight),
        ),
        (
            "supplemental_top_rate",
            shapes
                .iter()
                .any(|s| s.supplemental_top_rate != law.supplemental_top_rate),
        ),
        (
            "transportation_floor",
            shapes
                .iter()
                .any(|s| s.transportation_floor != law.transportation_floor),
        ),
    ];

    let unmoved: Vec<&str> = moved
        .iter()
        .filter(|(_, was)| !was)
        .map(|(name, _)| *name)
        .collect();
    assert!(
        unmoved.is_empty(),
        "no checkpoint moves {unmoved:?}. A lever the checkpoints never move is a lever the \
         browser is never required to implement, and the scenario page will report itself \
         verified while computing it wrongly or not at all. Add one to `checkpoint_policies`."
    );

    // And the list above is the whole of `PolicyShape` rather than the fields someone remembered.
    // Nine entries against nine fields; a tenth field added without a tenth entry fails here
    // rather than passing silently.
    assert_eq!(
        moved.len(),
        9,
        "PolicyShape has a field this test does not cover"
    );
}
