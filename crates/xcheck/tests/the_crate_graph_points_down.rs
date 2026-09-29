//! The workspace's dependency arrows, read off the manifests, all point down.
//!
//! Two shapes are refused, both from #501. A crate whose dev-dependency depends on it back is
//! legal — Cargo allows it because neither edge is a build one — but the crate is then compiled
//! twice, once as itself and once inside the dependency, and `cargo test -p dispersion` built
//! all of `project` to run five files. Four such pairs had accumulated, each with a comment
//! explaining it, and nothing refused the fifth. And `connect`, which reads publications, reached
//! every model crate through `bundle` for one README block. Each was a line in a `Cargo.toml` that
//! looked reasonable where it was written.
//!
//! Read from the manifests rather than from `cargo metadata` so the check needs no subprocess
//! and no network, in keeping with the workspace having no crates.io dependencies.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

/// A crate's workspace dependencies, by kind.
#[derive(Default)]
struct Edges {
    normal: BTreeSet<String>,
    dev: BTreeSet<String>,
}

/// Every workspace crate's path dependencies, from `crates/*/Cargo.toml`.
fn manifests() -> BTreeMap<String, Edges> {
    let crates = Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    let mut graph = BTreeMap::new();
    for entry in fs::read_dir(&crates)
        .expect("crates/ is readable")
        .flatten()
    {
        let manifest = entry.path().join("Cargo.toml");
        let Ok(text) = fs::read_to_string(&manifest) else {
            continue;
        };
        let name = entry.file_name().to_string_lossy().into_owned();
        graph.insert(name, parse(&text));
    }
    graph
}

/// The `path = "../<name>"` entries under `[dependencies]` and `[dev-dependencies]`.
fn parse(text: &str) -> Edges {
    let mut edges = Edges::default();
    let mut section = "";
    for line in text.lines().map(str::trim) {
        if line.starts_with('[') {
            section = line;
            continue;
        }
        let Some(target) = line
            .split("path = \"../")
            .nth(1)
            .and_then(|rest| rest.split('"').next())
        else {
            continue;
        };
        match section {
            "[dependencies]" => edges.normal.insert(target.to_string()),
            "[dev-dependencies]" => edges.dev.insert(target.to_string()),
            _ => false,
        };
    }
    edges
}

/// Every crate `from` reaches through normal dependencies, itself excluded.
fn closure(graph: &BTreeMap<String, Edges>, from: &str) -> BTreeSet<String> {
    let mut seen = BTreeSet::new();
    let mut stack: Vec<&str> = graph[from].normal.iter().map(String::as_str).collect();
    while let Some(next) = stack.pop() {
        if seen.insert(next.to_string()) {
            stack.extend(graph[next].normal.iter().map(String::as_str));
        }
    }
    seen
}

/// `(crate, dev-dependency)` pairs where the dev-dependency depends back on the crate.
fn dev_cycles(graph: &BTreeMap<String, Edges>) -> Vec<(String, String)> {
    let mut found = Vec::new();
    for (name, edges) in graph {
        for dev in &edges.dev {
            if closure(graph, dev).contains(name) {
                found.push((name.clone(), dev.clone()));
            }
        }
    }
    found
}

#[test]
fn the_manifests_were_read() {
    // A parser that found nothing would pass every assertion below.
    let graph = manifests();
    assert!(graph.len() >= 15, "{} crates", graph.len());
    assert!(graph["project"].normal.contains("dispersion"));
    assert!(graph["xcheck"].dev.contains("project"));
}

#[test]
fn no_dev_dependency_depends_back_on_the_crate_that_names_it() {
    assert_eq!(dev_cycles(&manifests()), Vec::<(String, String)>::new());
}

#[test]
fn the_cycle_check_finds_the_pair_it_was_written_for() {
    // The real graph before #501: `dispersion` dev-depended on `project`, which depends on it.
    let mut graph = manifests();
    graph
        .get_mut("dispersion")
        .unwrap()
        .dev
        .insert("project".to_string());
    assert_eq!(
        dev_cycles(&graph),
        vec![("dispersion".to_string(), "project".to_string())]
    );
}

#[test]
fn connect_reaches_no_model_crate() {
    // Retrieval and extraction reads publications with `spreadsheet`, names years with
    // `edfund-core`, and deflates the CPI check with `deflator`. Anything past those means a
    // change to the model recompiles the fetcher again.
    let reached = closure(&manifests(), "connect");
    let allowed: BTreeSet<String> = ["deflator", "edfund-core", "spreadsheet"]
        .map(String::from)
        .into();
    assert_eq!(reached, allowed);
}
