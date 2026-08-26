//! The crate half of the corpus cross-check.
//!
//! `web/tests/unit/corpusFigures.spec.ts` asserts that a corpus node agrees with
//! `crates/figures.json`. That is only worth having if the manifest agrees with the calculators,
//! and these are the assertions that say so. Split across the two languages deliberately: the
//! corpus check can then fail with "this node is stale" and mean it, rather than meaning "one of
//! the two ends moved and this one noticed first".
//!
//! # The registry invariants are here and not in the consumer
//!
//! A duplicated key, a key that does not name its owner, an owner that is not a directory — every
//! one of those is a defect in this file, and a check that lives in the consumer reports it as a
//! corpus problem to whoever is editing the corpus. #125's finding was sixteen checks that were
//! green against the defect they were written for; a check pointed at the wrong half of a seam is
//! the next thing along from that.

use std::collections::BTreeSet;
use std::path::Path;

use figures::{compute_all, Unit, CONTRACT_VERSION, FIGURES};

/// The pin, which is the whole reason [`figures::Figure::pinned`] exists.
///
/// A calculator change that moves a figure fails here, in the workspace that changed, naming the
/// figure and both values — rather than regenerating the manifest silently and surfacing three
/// steps later as a corpus node that disagrees with a number nobody knew had moved.
#[test]
fn the_manifest_reproduces_its_pins() {
    for entry in compute_all() {
        let f = entry.figure;
        let drift = (entry.value - f.pinned).abs();
        assert!(
            drift <= f.tolerance,
            "{}: computes {} against a pin of {} — {drift} away, tolerating {}.\n\
             If the calculator is right, move the pin and regenerate crates/figures.json; the \
             corpus nodes bound to this key will then say which prose has to change.",
            f.key,
            entry.value,
            f.pinned,
            f.tolerance,
        );
    }
}

/// The same tree produces the same bytes, twice in one process.
///
/// Not a formality. `regime_diff::recognized_valuation::from_abstract` returns a `HashMap`, which
/// seeds a fresh hasher per instance, so iterating it to sum 609 districts' valuations gave a
/// different order each time — and floating-point addition is not associative. Three figures'
/// last three digits were a coin flip, which made a *generated, committed* artefact one that
/// could not be regenerated: `mise run //:generated` red on a clean tree, which is #124 again in
/// a new place. Sorting the sum fixed it and this is what holds it.
#[test]
fn the_manifest_is_stable() {
    assert_eq!(
        figures::manifest(),
        figures::manifest(),
        "two runs in one process disagree, so the committed artefact cannot be regenerated"
    );
}

/// A count is a count. Nothing below rounds one, so a fractional pin is a unit that is wrong.
#[test]
fn a_count_is_whole_and_exact() {
    for f in FIGURES {
        if f.unit != Unit::Count {
            continue;
        }
        assert!(
            f.pinned.fract() == 0.0,
            "{}: pinned at {}, which is not a whole number of things",
            f.key,
            f.pinned
        );
        assert!(
            f.tolerance == 0.0,
            "{}: a count with a tolerance of {} — a count either is the number or is not",
            f.key,
            f.tolerance
        );
    }
}

/// A share is small enough that a percentage typed in its place would be obvious.
///
/// The failure this forecloses is bundle contract `35.0.0`'s: two fields with the same name, 100x
/// apart, both `f64` and neither saying which it was. Here it would put `28.1` in the manifest
/// where the consumer expects `0.281`, and the consumer would read a corpus's `28.1%` as agreeing
/// with a figure a hundred times its size.
///
/// # Why the ceiling is not one
///
/// It was `0.0..=1.0`, which is right for a *proportion* and wrong for the other thing prose
/// writes with a percent sign: a **growth rate**. Ohio's operating expenditure per pupil rose
/// **116.8%** in nominal terms between FY2000 and FY2022, which is `1.168` and is not a mistake.
/// `Unit::Ratio` is not the escape either — `numerals()` rejects a `%` on a ratio, by design, so
/// a ratio binding could not match the sentence the corpus actually writes.
///
/// So the ceiling states the invariant the guard is really for, rather than a proxy for it: a pin
/// must be far enough below a percentage that the two cannot be confused. The second assertion is
/// what keeps that honest — it checks the guard's own premise, that **every** share here is large
/// enough for its 100x typo to land above the ceiling. Measured over the manifest at the time of
/// writing: the smallest share is `0.048`, whose mistyped form is `4.8`. Add a share below
/// [`SHARE_FLOOR`] and this test fails and says so, rather than quietly losing its teeth.
#[test]
fn a_share_cannot_be_confused_with_a_percentage() {
    /// A growth rate may exceed one. Nothing this manifest carries approaches two.
    const SHARE_CEILING: f64 = 2.0;
    /// Below this, a share's own 100x typo would slip under the ceiling undetected.
    const SHARE_FLOOR: f64 = SHARE_CEILING / 100.0;

    for f in FIGURES {
        if f.unit != Unit::Share {
            continue;
        }
        // Signed shares exist — a decline is a share too — so the magnitude is what is bounded.
        let magnitude = f.pinned.abs();
        assert!(
            magnitude <= SHARE_CEILING,
            "{}: pinned at {}, which is past {SHARE_CEILING} — is it a percentage?",
            f.key,
            f.pinned
        );
        assert!(
            magnitude > SHARE_FLOOR,
            "{}: pinned at {}, small enough that typing it as a percentage would give {}, \
             which is under the {SHARE_CEILING} ceiling and would pass. Either it is not a \
             share, or the ceiling above no longer holds.",
            f.key,
            f.pinned,
            magnitude * 100.0
        );
    }
}

/// Keys are unique, and each names the directory of the crate that owns it.
///
/// The prefix is not decoration. It is what lets the consumer report "this node cites
/// `crates/project` and binds a `regime-diff` figure" without needing the manifest's `owner` field
/// to be trustworthy independently of the key.
#[test]
fn every_key_is_unique_and_names_its_owner() {
    let mut seen = BTreeSet::new();
    for f in FIGURES {
        assert!(seen.insert(f.key), "{}: two figures share this key", f.key);

        let directory = f
            .owner
            .strip_prefix("crates/")
            .unwrap_or_else(|| panic!("{}: owner {:?} is not under crates/", f.key, f.owner));
        assert!(
            f.key.starts_with(&format!("{directory}/")),
            "{}: owned by {} and so should be keyed `{directory}/…`",
            f.key,
            f.owner
        );
        assert!(
            f.key
                .chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-' || c == '/'),
            "{}: keys are lower-case kebab so a corpus node can hold one without quoting",
            f.key
        );
    }
}

/// The owner is a crate that exists. A figure attributed to a directory that is not there is a
/// citation the corpus would inherit and nobody could follow.
#[test]
fn every_owner_is_a_crate_in_this_workspace() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("crates/figures sits inside crates/")
        .parent()
        .expect("crates/ sits inside the repository");
    for f in FIGURES {
        let manifest = root.join(f.owner).join("Cargo.toml");
        assert!(
            manifest.exists(),
            "{}: owned by {}, which has no Cargo.toml at {}",
            f.key,
            f.owner,
            manifest.display()
        );
    }
}

/// A label is a sentence somebody reading the artefact can act on, and is writable unescaped.
#[test]
fn every_string_is_safe_to_write_unescaped() {
    for f in FIGURES {
        assert!(
            f.label.len() > 20,
            "{}: {:?} does not say what the figure is",
            f.key,
            f.label
        );
        for (what, text) in [("key", f.key), ("owner", f.owner), ("label", f.label)] {
            assert!(
                !text.contains(['"', '\\']) && !text.chars().any(char::is_control),
                "{}: the {what} carries a character the manifest writer cannot escape",
                f.key
            );
        }
    }
}

/// The document parses as JSON, declares its contract, and carries every figure once.
///
/// A hand-rolled writer that emits something no consumer can read is the failure this forecloses,
/// and the corpus check would report it as "no figures are bound" — a green-looking nothing.
#[test]
fn the_document_is_readable_and_complete() {
    let json = figures::manifest();
    assert!(json.starts_with("{\n"), "the document is an object");
    assert!(
        json.ends_with("}\n"),
        "the document is closed and newline-terminated"
    );
    assert!(
        json.contains(&format!("\"contract\": \"{CONTRACT_VERSION}\"")),
        "the document states its contract version"
    );
    assert_eq!(
        json.matches("{\"key\": ").count(),
        FIGURES.len(),
        "every figure is written exactly once"
    );
    for f in FIGURES {
        assert!(
            json.contains(&format!("\"key\": \"{}\"", f.key)),
            "{} is missing from the manifest",
            f.key
        );
    }
    assert_eq!(
        json.matches('{').count(),
        json.matches('}').count(),
        "the writer closed every object it opened"
    );
    assert_eq!(
        json.matches('[').count(),
        json.matches(']').count(),
        "the writer closed every array it opened"
    );
}
