//! The shapes the feed publishes.
//!
//! Nothing here serializes; [`crate::serialize`] does that, and the split is the point. These
//! type definitions and the code that writes them out used to sit in one 4,200-line file, with
//! 357 lines of them stranded on the far side of a 1,116-line test module from the emitters
//! that read them.
//!
//! Re-exported flat from the crate root, so `bundle::District` still resolves — the layout
//! changed, the public API did not.

pub mod district;
pub mod statewide;
pub mod year;

pub use district::*;
pub use statewide::*;
pub use year::*;
