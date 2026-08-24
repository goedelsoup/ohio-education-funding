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

/// A share is a fraction of one, so a pin above one is a percentage that forgot to divide.
///
/// The failure this forecloses is bundle contract `35.0.0`'s: two fields with the same name, 100x
/// apart, both `f64` and neither saying which it was. Here it would put `28.1` in the manifest
/// where the consumer expects `0.281`, and the consumer would read a corpus's `28.1%` as agreeing
/// with a figure a hundred times its size.
#[test]
fn a_share_is_a_fraction_of_one() {
    for f in FIGURES {
        if f.unit != Unit::Share {
            continue;
        }
        assert!(
            (0.0..=1.0).contains(&f.pinned),
            "{}: pinned at {}, which is not a share of one — is it a percentage?",
            f.key,
            f.pinned
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
