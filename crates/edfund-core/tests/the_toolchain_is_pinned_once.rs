//! The Rust version is written in three files, and they must say the same thing.
//!
//! # Why three
//!
//! `crates/rust-toolchain.toml` is what rustup reads, and so what the CI `crates` job compiles and
//! lints with. `mise.toml` is what a local run uses: mise exports `RUSTUP_TOOLCHAIN` for its own
//! pin, which outranks the toolchain file, so the file cannot stand in for it. And
//! `rust-version` in `crates/Cargo.toml` is the floor Cargo refuses to build under and clippy's
//! MSRV-aware lints read.
//!
//! Before #496 only the second existed. The runner ignored it and used whatever stable its image
//! shipped, and clippy went red twice with no commit behind it.
//!
//! # Why this is a test
//!
//! For the reason `the_two_medians_have_one_definition_each.rs` gives: `.github/workflows/ci.yml`
//! runs the bare cargo commands rather than the mise gate, so a check in a mise task would never
//! run on the runner. A test runs wherever `cargo test` does.

use std::fs;
use std::path::{Path, PathBuf};

/// The workspace directory, `crates/`.
fn workspace() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("the crate sits inside the workspace")
        .to_path_buf()
}

fn read(path: &Path) -> String {
    fs::read_to_string(path).unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()))
}

/// The quoted value of the first line in `text` that starts with `key`, after the key has been
/// followed by `=` or, for mise's inline table, by `= { version =`.
fn quoted_after(text: &str, key: &str) -> Option<String> {
    text.lines().find_map(|line| {
        let rest = line.trim_start().strip_prefix(key)?.trim_start();
        let rest = rest.strip_prefix('=')?.trim_start();
        let rest = rest
            .strip_prefix('{')
            .map(|inner| {
                inner
                    .trim_start()
                    .strip_prefix("version")?
                    .trim_start()
                    .strip_prefix('=')
            })
            .unwrap_or(Some(rest))?
            .trim_start();
        let rest = rest.strip_prefix('"')?;
        Some(rest[..rest.find('"')?].to_owned())
    })
}

#[test]
fn the_toolchain_file_mise_and_the_workspace_name_one_version() {
    let root = workspace();
    let toolchain = quoted_after(&read(&root.join("rust-toolchain.toml")), "channel")
        .expect("rust-toolchain.toml names a channel");
    let mise = quoted_after(&read(&root.join("../mise.toml")), "rust")
        .expect("mise.toml pins rust in [tools]");
    let floor = quoted_after(&read(&root.join("Cargo.toml")), "rust-version")
        .expect("[workspace.package] declares rust-version");

    assert_eq!(
        toolchain, mise,
        "crates/rust-toolchain.toml says {toolchain} and mise.toml says {mise}: CI and a local run \
         would lint on different compilers"
    );
    assert_eq!(
        toolchain, floor,
        "crates/rust-toolchain.toml says {toolchain} and the workspace rust-version says {floor}"
    );
}

#[test]
fn the_toolchain_file_installs_the_components_ci_runs() {
    let text = read(&workspace().join("rust-toolchain.toml"));
    let components = text
        .lines()
        .find(|line| line.trim_start().starts_with("components"))
        .expect("rust-toolchain.toml lists components");
    for needed in ["rustfmt", "clippy"] {
        assert!(
            components.contains(&format!("\"{needed}\"")),
            "the CI job runs cargo {needed} and the toolchain file does not install it"
        );
    }
}

/// A `[workspace.package]` field reaches a crate only if the crate opts in, so a member that does
/// not is built with no floor at all.
#[test]
fn every_member_inherits_the_floor() {
    let root = workspace();
    let manifest = read(&root.join("Cargo.toml"));
    let members: Vec<&str> = manifest
        .split_once("members = [")
        .and_then(|(_, rest)| rest.split_once(']'))
        .expect("the workspace lists its members")
        .0
        .split(',')
        .map(|m| m.trim().trim_matches('"'))
        .filter(|m| !m.is_empty())
        .collect();

    // A parse that finds no members would pass the loop below vacuously.
    assert!(members.len() >= 10, "found only {members:?}");

    for member in members {
        let text = read(&root.join(member).join("Cargo.toml"));
        assert!(
            text.lines()
                .any(|l| l.trim() == "rust-version.workspace = true"),
            "{member}/Cargo.toml does not inherit rust-version from the workspace"
        );
    }
}
