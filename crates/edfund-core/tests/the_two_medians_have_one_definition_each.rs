//! Nothing in this workspace may define its own median or its own correlation.
//!
//! # Why this is a test and not a grep in the gate
//!
//! Because the gate is not what runs in CI. `mise run //crates:gate` fans out fmt, clippy, test
//! and doc, but `.github/workflows/ci.yml` invokes the bare cargo commands itself, so a shell
//! step added to the mise task would be green on this machine and absent on the runner. A test
//! runs wherever `cargo test` does.
//!
//! # What it is for
//!
//! Before #494 the workspace held two median conventions under one name. `dispersion::median`
//! interpolated between the two middle observations of an even sample; six private helpers in
//! `crates/project`, and four more modules inline, took the upper of them. The corpus binds
//! medians at full `f64` precision, so two figures on one panel could disagree with each other
//! and nothing on the page said which median either meant.
//!
//! Naming both conventions in [`edfund_core::stats`] fixes the ten copies that existed. This
//! fixes the eleventh: a private helper is three lines and reaching for one is the obvious thing
//! to do, so the reason not to has to be enforced rather than remembered.
//!
//! # Why the count is asserted
//!
//! A scan that walks a directory tree and finds nothing is indistinguishable from a scan that
//! walks nothing. The floor is well under the current file count so that adding a crate does not
//! move it, and far enough above zero that a broken walk fails loudly.

use std::fs;
use std::path::{Path, PathBuf};

/// The lowest number of Rust sources a working walk of `crates/` can reach. 369 at the time of
/// writing; the floor is slack so that deleting a module does not fail this test for the wrong
/// reason.
const AT_LEAST: usize = 300;

/// The function names that may exist in exactly one place.
///
/// Spelled as bare identifiers rather than with the keyword in front of them so that this file is
/// not its own first violation.
const RESERVED: [&str; 3] = ["median", "upper_middle", "correlation"];

/// The one module allowed to define them, and this file, which has to name them to look for them.
const EXEMPT: [&str; 2] = [
    "edfund-core/src/stats.rs",
    "edfund-core/tests/the_two_medians_have_one_definition_each.rs",
];

/// Every `.rs` file under `crates/`, `target/` aside.
fn sources() -> Vec<PathBuf> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("the crate sits inside the workspace")
        .to_path_buf();
    let mut found = Vec::new();
    let mut pending = vec![root];
    while let Some(dir) = pending.pop() {
        for entry in fs::read_dir(&dir).expect("a readable workspace directory") {
            let path = entry.expect("a readable directory entry").path();
            if path.is_dir() {
                if path.file_name().is_some_and(|name| name != "target") {
                    pending.push(path);
                }
            } else if path.extension().is_some_and(|ext| ext == "rs") {
                found.push(path);
            }
        }
    }
    found
}

/// The identifier a `fn` keyword introduces, where `at` is the byte index of the `f`.
fn declared_name(line: &str, at: usize) -> Option<&str> {
    let before_is_word = line[..at]
        .chars()
        .next_back()
        .is_some_and(|c| c.is_alphanumeric() || c == '_');
    if before_is_word {
        return None;
    }
    let rest = line[at + 3..].trim_start();
    let end = rest
        .find(|c: char| !c.is_alphanumeric() && c != '_')
        .unwrap_or(rest.len());
    (end > 0).then(|| &rest[..end])
}

/// **One definition each, workspace-wide.**
#[test]
fn no_crate_defines_its_own_median_or_correlation() {
    let files = sources();
    assert!(
        files.len() >= AT_LEAST,
        "the walk reached {} sources, which is too few to have looked anywhere",
        files.len()
    );

    let keyword = "fn ";
    let mut violations: Vec<String> = Vec::new();
    let mut scanned = 0_usize;
    for path in &files {
        let shown = path.to_string_lossy().replace('\\', "/");
        if EXEMPT.iter().any(|tail| shown.ends_with(tail)) {
            continue;
        }
        scanned += 1;
        let body = fs::read_to_string(path).expect("a readable Rust source");
        for (number, line) in body.lines().enumerate() {
            if line.trim_start().starts_with("//") {
                continue;
            }
            for (at, _) in line.match_indices(keyword) {
                if declared_name(line, at).is_some_and(|name| RESERVED.contains(&name)) {
                    violations.push(format!("{shown}:{}: {}", number + 1, line.trim()));
                }
            }
        }
    }

    assert_eq!(
        scanned + EXEMPT.len(),
        files.len(),
        "every exemption should have matched exactly one file"
    );
    assert!(
        violations.is_empty(),
        "these define a statistic edfund_core::stats already names — call \
         median_interpolated, median_upper_middle or correlation instead, and say at the call \
         site which convention the figure was computed on:\n{}",
        violations.join("\n")
    );
}
