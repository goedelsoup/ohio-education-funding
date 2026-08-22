//! Write the feed to stdout.
//!
//! Everything this used to hold lives in [`bundle::build`] now, so that `connect` and the tests
//! can call it rather than parse its output.

fn main() {
    print!("{}", bundle::build::build().to_json());
}
