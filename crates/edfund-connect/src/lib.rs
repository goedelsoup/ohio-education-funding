//! The `edfund-connect` command, and the README index it regenerates.
//!
//! The binary's retrieval and extraction commands are thin calls into [`connect`]. It lives in
//! its own package because one of its commands does not belong there: [`index`] writes the
//! `bundle-status` REGEN block by running [`bundle::build::build`], and while that module sat in
//! `connect` the crate that reads the department's publications depended on every model crate
//! through `bundle` — and recompiled whenever any of them changed (#501). The package is named
//! for the binary so the REGEN markers, which name their generator as `edfund-connect <cmd>`,
//! still name the thing that owns them.

#![forbid(unsafe_code)]

pub mod index;
