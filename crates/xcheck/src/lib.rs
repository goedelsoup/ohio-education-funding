//! Cross-checks: one crate's arithmetic held against another crate's fixtures.
//!
//! There is nothing in the library. The crate exists for `tests/`, where each file checks a
//! crate against data another crate reads — the millage calculator against Table SD-1, the
//! local capacity measure against the panel, the report card against the department's model.
//! Those tests used to live in the crate under test, reaching the other as a dev-dependency, and
//! where that crate sat above the one under test Cargo compiled the leaf twice (#501). A test
//! belongs here when the crate it needs depends, directly or not, on the crate it checks.

#![forbid(unsafe_code)]
