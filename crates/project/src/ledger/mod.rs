//! The appropriations ledger: what the General Assembly set aside, line by line and year by year.
//!
//! Four modules, and the boundary they sit behind is the point of the directory.
//!
//! | Module | Question it answers |
//! |---|---|
//! | [`appropriations`] | how much a line was given, FY2010 to FY2027 |
//! | [`session_laws`] | the same, FY1998 to FY2001, from a different publisher |
//! | [`line_origins`] | how old a line is, and which act established it |
//! | [`budget_analysis`] | what a line's earmarks are, and whether they survived the legislature |
//!
//! # The invariant: the ledger knows nothing about districts
//!
//! **Nothing under `ledger` may reference anything else in this crate.** Not [`mod@crate::panel`],
//! not [`mod@crate::policy`], [`mod@crate::report`], [`mod@crate::drafts`], [`mod@crate::series`],
//! [`mod@crate::outcomes`], or [`mod@crate::crosswalk`] — and not whatever cluster-A module is added
//! after this sentence was written, which is why the test
//! below checks the shape of the path rather than a list of names.
//!
//! It already held by accident. The crate is two clusters with **zero** production edges between
//! them: the district panel and its simulator on one side, this ledger on the other. Writing the
//! rule down is what turns "nobody happened to" into something a reviewer can hold a patch to.
//!
//! # Why this is not a crate yet
//!
//! Three reasons, recorded so the question is not reopened without new facts.
//!
//! `bundle` consumes both clusters, so splitting shrinks nobody's dependency graph — it is the
//! largest consumer and would depend on both halves. The binding that matters is *documentary*:
//! `tests/the_statute_behind_the_weights.rs` asserts the statute text agrees with [`mod@crate::panel`]'s
//! constants, and the appropriation figures those weights are prorated against live here, so a
//! crate boundary would put the most valuable check in the suite across it. And this is well under
//! the ~2,000 source lines named below, where a crate buys ceremony rather than clarity.
//!
//! **Split it for real when either becomes true:** the ledger passes ~2,000 lines, or a second
//! consumer wants appropriations without compiling the district model. Today three of `project`'s
//! five dependents — `regime-diff`, `local-capacity`, `scenario-delta` — use only [`mod@crate::panel`]
//! and its neighbours, and pay for 1.6 MB of embedded ledger fixtures they never call. That is the
//! cost being carried, and the directory is what makes paying it off a `git mv`.

pub mod appropriations;
pub mod budget_analysis;
pub mod line_origins;
pub mod session_laws;

#[cfg(test)]
mod tests {
    /// The invariant this directory exists to state, checked rather than trusted.
    ///
    /// A path test rather than a name test. Forbidding `crate::panel` by name would pass the day
    /// someone reaches for a module invented after this was written; requiring that every absolute
    /// path inside the ledger begins `crate::ledger::` closes the rule over modules that do not
    /// exist yet. `super::super::` is the same reach spelled relatively, so it goes too.
    ///
    /// Comment lines are dropped before the search, because a comment cannot create an edge and
    /// both this file's prose and this docstring have to be able to name what they forbid. The
    /// three needles are spelled in halves for the same reason: this file is one of the four it
    /// searches, and a literal `crate` followed by two colons would match itself.
    ///
    /// The sources are read at compile time, which costs the test binary a copy of five files it
    /// already contains and costs the library nothing.
    #[test]
    fn the_ledger_reaches_nothing_outside_itself() {
        const SOURCES: [(&str, &str); 5] = [
            ("mod.rs", include_str!("mod.rs")),
            ("appropriations.rs", include_str!("appropriations.rs")),
            ("budget_analysis.rs", include_str!("budget_analysis.rs")),
            ("session_laws.rs", include_str!("session_laws.rs")),
            ("line_origins.rs", include_str!("line_origins.rs")),
        ];
        const ABSOLUTE: &str = concat!("crate", "::");
        const OWN: &str = concat!("ledger", "::");
        const OUTWARD: &str = concat!("super", "::", "super", "::");

        for (name, source) in SOURCES {
            let body = source
                .lines()
                .filter(|line| !line.trim_start().starts_with("//"))
                .collect::<Vec<&str>>()
                .join("\n");

            for (index, _) in body.match_indices(ABSOLUTE) {
                let rest = &body[index + ABSOLUTE.len()..];
                assert!(
                    rest.starts_with(OWN),
                    "{name} reaches out of the ledger: {ABSOLUTE}{}",
                    rest.split(|c: char| !c.is_alphanumeric() && c != '_')
                        .next()
                        .unwrap_or_default()
                );
            }
            assert!(
                !body.contains(OUTWARD),
                "{name} reaches out of the ledger through {OUTWARD}"
            );
        }
    }
}
