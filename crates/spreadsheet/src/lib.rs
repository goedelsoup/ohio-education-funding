//! Read the department's published workbooks, with nothing underneath.
//!
//! The Ohio Department of Education and Workforce publishes its funding data as Excel
//! workbooks. Reading them needs a zip reader, a DEFLATE decompressor, and an XML parser. This
//! crate is all three, in about a thousand lines, because the workspace takes no dependencies.
//!
//! # Why that constraint is kept here too
//!
//! [`edfund_core`](../../edfund-core/) explains the rule for calculators: a
//! [`scenario`](../../../.yidam/corpus/scenario/) result should be reproducible years from now
//! without a dependency resolution succeeding first. The rule matters at least as much on the
//! retrieval side, where the alternative is an extraction pipeline that stops working when a
//! transitive dependency yanks a version — and an extraction pipeline that will not run is a
//! corpus that cannot be refreshed.
//!
//! This layer knows about *formats*. It knows nothing about Ohio: no IRNs, no fiscal years, no
//! suppressed-count conventions. Those live in [`connect`](../../connect/), which is where the
//! department-specific knowledge belongs and where it can be revised when a publication
//! changes shape.
//!
//! ```no_run
//! use spreadsheet::Workbook;
//!
//! let bytes = std::fs::read("fy27-calculator.xlsx")?;
//! let book = Workbook::open(bytes)?;
//! for row in book.rows("Base_Cost")?.iter().take(5) {
//!     println!("{row:?}");
//! }
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

#![forbid(unsafe_code)]

pub mod inflate;
pub mod xlsx;
pub mod xml;
pub mod zip;

pub use xlsx::{column_index, Workbook, XlsxError};
pub use zip::{Archive, ZipError};
