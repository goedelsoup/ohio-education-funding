//! Regenerating the `REGEN` blocks the READMEs carry.
//!
//! Six READMEs held blocks reading `_Run `yidam corpus-index` to populate._` for fourteen
//! phases. There is no `yidam` binary — not in this repository and not in the vendored prelude —
//! so those were instructions to run something that does not exist, sitting under headings that
//! promised content.
//!
//! Everything here is derived from the repository itself, so a stale block is a failing check
//! rather than a thing someone notices. That matters more than it sounds: the test counts in
//! `crates/README.md` drifted by 41 across four phases and nobody saw, because a number in
//! prose has nothing checking it.
//!
//! # What is not generated, and why
//!
//! One block describes a thing that does not exist: a semantic index over the corpus, never built
//! because the corpus fits in context. It is replaced with a sentence saying so. Generating a
//! plausible-looking status for a system that is not there would be worse than the empty block
//! was — but the sentence still counts the corpus, because a status that is a string literal
//! regenerates to itself and can therefore be wrong forever. It was, by sixteen nodes.
//!
//! # Prose is where this drifts now
//!
//! The blocks are safe and the paragraphs beside them are not. `crates/connect/README.md` carried
//! a hand-written connector table claiming nine connectors and three wired while the registry held
//! thirteen and eleven; `crates/README.md` described the same nine in prose. Neither was reachable
//! by the guard, because the guard reads markers. The table is now a `connector-registry` block,
//! generated from [`crate::registry::CONNECTORS`] itself.

use std::collections::BTreeMap;
use std::fs;
use std::io;
use std::path::Path;

/// A block that could not be regenerated.
#[derive(Debug)]
pub enum IndexError {
    /// A file named a block this tool does not know how to fill.
    UnknownBlock {
        /// The command named in the marker.
        command: String,
        /// Where it was found.
        file: String,
    },
    /// A filesystem operation failed.
    Io(io::Error),
}

impl core::fmt::Display for IndexError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::UnknownBlock { command, file } => {
                write!(f, "{file}: no generator for REGEN block {command:?}")
            }
            Self::Io(cause) => write!(f, "{cause}"),
        }
    }
}

impl std::error::Error for IndexError {}

impl From<io::Error> for IndexError {
    fn from(cause: io::Error) -> Self {
        Self::Io(cause)
    }
}

/// A README carrying at least one block.
pub const DOCUMENTS: &[&str] = &[
    "README.md",
    "crates/README.md",
    "crates/connect/README.md",
    "web/README.md",
    "agents/README.md",
    ".yidam/corpus/README.md",
    ".yidam/catalog/README.md",
    ".yidam/skills/README.md",
];

/// A leading `key: value` line — the corpus stores `class:` and `label:` that way.
fn field(text: &str, key: &str) -> Option<String> {
    text.lines()
        .find_map(|line| line.strip_prefix(&format!("{key}:")))
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

/// A `key = "value"` line, which is how TOML says the same thing.
fn toml_field(text: &str, key: &str) -> Option<String> {
    text.lines()
        .find_map(|line| line.trim().strip_prefix(&format!("{key} = ")))
        .map(|value| value.trim().trim_matches('"').to_string())
        .filter(|value| !value.is_empty())
}

/// How many *other corpus nodes* this node's text references.
///
/// The exact mirror of the inbound count in [`corpus_index`], and deliberately so. This used to be
/// `text.match_indices("](").count()` — every markdown link in the file, which is a different
/// quantity in three ways at once: it counted prose citations only and never the structured
/// `links:` list, it counted links to the catalog and to `crates/` as though they were edges, and
/// it counted the same target twice if a node mentioned it twice.
///
/// The effect was that the column measured how *chatty* a node's prose was rather than what it
/// pointed at. Converting `scenario/guarantee-phase-out` from a prose links block to a structured
/// one moved its edges out of the text and its reported out-degree fell from 15 to 4: the node
/// became correct and the number got worse, which is the signature of a metric measuring the
/// wrong thing.
///
/// Matching on `<slug>.yml` catches both forms, because a structured `target:` and a markdown link
/// both contain the filename. Verified free of suffix collisions across all 62 slugs — no node's
/// filename ends with another's.
fn outgoing_links(text: &str, slugs: &[&str], self_slug: &str) -> usize {
    slugs
        .iter()
        .filter(|slug| **slug != self_slug && text.contains(&format!("{slug}.yml")))
        .count()
}

fn read(root: &Path, relative: &str) -> String {
    fs::read_to_string(root.join(relative)).unwrap_or_default()
}

/// How many records a directory holds: every file with `extension`, less any `README`.
///
/// The index is not one of the things it indexes, and every directory here keeps its own.
fn records(root: &Path, directory: &str, extension: &str) -> usize {
    let Ok(entries) = fs::read_dir(root.join(directory)) else {
        return 0;
    };
    entries
        .flatten()
        .filter(|entry| {
            let path = entry.path();
            path.extension().is_some_and(|e| e == extension)
                && path.file_stem().is_some_and(|stem| stem != "README")
        })
        .count()
}

/// Every corpus node, as `(class, slug, path, text)`.
fn corpus_nodes(root: &Path) -> Vec<(String, String, String, String)> {
    let mut out = Vec::new();
    let Ok(classes) = fs::read_dir(root.join(".yidam/corpus")) else {
        return out;
    };
    for class in classes.flatten() {
        if !class.path().is_dir() {
            continue;
        }
        let class_name = class.file_name().to_string_lossy().into_owned();
        let Ok(nodes) = fs::read_dir(class.path()) else {
            continue;
        };
        for node in nodes.flatten() {
            let path = node.path();
            if path.extension().is_some_and(|e| e == "yml") {
                let slug = path
                    .file_stem()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .into_owned();
                let relative = format!(".yidam/corpus/{class_name}/{slug}.yml");
                let text = fs::read_to_string(&path).unwrap_or_default();
                out.push((class_name.clone(), slug, relative, text));
            }
        }
    }
    out.sort_by(|a, b| (&a.0, &a.1).cmp(&(&b.0, &b.1)));
    out
}

fn corpus_index(root: &Path) -> String {
    let nodes = corpus_nodes(root);
    // Inbound links are counted by scanning every node for a reference to each slug, which is
    // O(n²) over 58 files and instant. A node nothing points at is the thing worth surfacing.
    let mut inbound: BTreeMap<&str, usize> = BTreeMap::new();
    for (_, slug, _, _) in &nodes {
        let needle = format!("{slug}.yml");
        let count = nodes
            .iter()
            .filter(|(_, other, _, text)| other != slug && text.contains(&needle))
            .count();
        inbound.insert(slug.as_str(), count);
    }

    let slugs: Vec<&str> = nodes.iter().map(|(_, slug, _, _)| slug.as_str()).collect();

    let mut out = String::from("| Node | Class | Label | Out | In |\n|---|---|---|--:|--:|\n");
    for (class, slug, relative, text) in &nodes {
        let label = field(text, "label").unwrap_or_else(|| slug.clone());
        let path = relative.trim_start_matches(".yidam/corpus/");
        out.push_str(&format!(
            "| [`{slug}`]({path}) | {class} | {label} | {} | {} |\n",
            outgoing_links(text, &slugs, slug),
            inbound.get(slug.as_str()).copied().unwrap_or(0)
        ));
    }
    let orphans = inbound.values().filter(|count| **count == 0).count();
    out.push_str(&format!(
        "\n{} nodes across {} classes, and **{orphans} with nothing pointing at them** — counting \
         a citation in somebody's prose as pointing. Whether that is a gap depends on the class: \
         `web/tests/unit/reachability.spec.ts` holds every node to having an inbound *edge* and \
         exempts `draft-legislation`, where a node with nothing pointing at it is the design.\n",
        nodes.len(),
        nodes
            .iter()
            .map(|(class, _, _, _)| class)
            .collect::<std::collections::BTreeSet<_>>()
            .len()
    ));
    out
}

/// Every relationship a node declares on an edge to another node, and whether its class declared it.
///
/// # The number this replaces, and why it had to be generated
///
/// Sixteen ontology files, the corpus README and `web/src/lib/schema/corpus.ts` each carried the
/// same hand-written measurement — "90 relationships in use against 65 declared", "46 of the 90
/// are undeclared", "a third of the graph's edges". It was the evidence for `edge_policy:
/// characteristic`, and by the time a review re-measured it the corpus had 152 relationships and
/// half its edges were undeclared. The argument survived; every number in it was wrong, in
/// eighteen places at once, and the generated node index two screens below had stayed current the
/// whole time.
///
/// # What counts as an edge here
///
/// A `links:` entry whose target is another node in this corpus — so `instance-of`, which points
/// at an ontology class, and `sourced-from`, which points at the catalog, are both out. They are
/// structural rather than semantic, and counting them would inflate the undeclared share with
/// edges no class would ever declare.
fn edge_vocabulary(root: &Path) -> String {
    let mut used: BTreeMap<String, usize> = BTreeMap::new();
    let mut undeclared = 0usize;
    let mut total = 0usize;

    let ontologies = ontology_edges(root);
    let mut declared: std::collections::BTreeSet<(String, String)> =
        std::collections::BTreeSet::new();
    for (class, relationships) in &ontologies {
        for relationship in relationships {
            declared.insert((class.clone(), relationship.clone()));
        }
    }

    for (class, _, _, text) in corpus_nodes(root) {
        for (target, relationship) in node_edges(&text) {
            if !target.ends_with(".yml") || target.ends_with(".ont.yml") {
                continue;
            }
            total += 1;
            *used.entry(relationship.clone()).or_default() += 1;
            if !declared.contains(&(class.clone(), relationship)) {
                undeclared += 1;
            }
        }
    }

    let singletons = used.values().filter(|count| **count == 1).count();
    let distinct = used.len();
    let declared_total: usize = ontologies.values().map(Vec::len).sum();
    let share = (undeclared * 100).checked_div(total).unwrap_or(0);

    let mut out = String::from("| Measure | Count |\n|---|--:|\n");
    out.push_str(&format!("| edges between nodes | {total} |\n"));
    out.push_str(&format!("| distinct relationships in use | {distinct} |\n"));
    out.push_str(&format!(
        "| relationships declared across every class | {declared_total} |\n"
    ));
    out.push_str(&format!(
        "| edges whose relationship its class does not declare | {undeclared} |\n"
    ));
    out.push_str(&format!(
        "| relationships used exactly once | {singletons} |\n"
    ));
    out.push_str(&format!(
        "\n**{share}% of edges use a relationship the class does not declare**, and \
         {singletons} of the {distinct} relationships in use are used a single time. That is \
         the case for `edge_policy: characteristic`: closing the vocabulary would reject those \
         edges or require {singletons} declarations that each describe one link.\n"
    ));
    out
}

/// One `links:` entry per pair, as `(target, relationship)`.
///
/// Positional rather than parsed: an entry opens at `  - ` and its keys sit at four spaces, while
/// a `note:` body sits at six. That is the same granularity [`claim_audit`] attributes fields at,
/// and it needs no parser in a crate that has deliberately avoided acquiring one.
fn node_edges(text: &str) -> Vec<(String, String)> {
    let mut out = Vec::new();
    let Some(start) = text.find("\nlinks:\n") else {
        return out;
    };
    let (mut target, mut relationship) = (None, None);
    for line in text[start + 1..].lines().skip(1) {
        if !line.starts_with(' ') && !line.trim().is_empty() {
            break;
        }
        if line.starts_with("  - ") {
            if let (Some(t), Some(r)) = (target.take(), relationship.take()) {
                out.push((t, r));
            }
        }
        let key = line.trim_start();
        let indent = line.len() - key.len();
        if indent > 4 && !line.starts_with("  - ") {
            continue;
        }
        if let Some(value) = key
            .strip_prefix("- target:")
            .or_else(|| key.strip_prefix("target:"))
        {
            target = Some(value.trim().to_string());
        } else if let Some(value) = key
            .strip_prefix("- relationship:")
            .or_else(|| key.strip_prefix("relationship:"))
        {
            relationship = Some(value.trim().to_string());
        }
    }
    if let (Some(t), Some(r)) = (target, relationship) {
        out.push((t, r));
    }
    out
}

/// The out-direction relationships each ontology class declares.
///
/// `direction: in` names an edge written on the *other* node, so it is not something a node of
/// this class declares and must not count toward what this class permits.
fn ontology_edges(root: &Path) -> BTreeMap<String, Vec<String>> {
    let mut out: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let Ok(entries) = fs::read_dir(root.join(".yidam/corpus")) else {
        return out;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let name = path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .into_owned();
        let Some(class) = name.strip_suffix(".ont.yml") else {
            continue;
        };
        let text = fs::read_to_string(&path).unwrap_or_default();
        let Some(start) = text.find("\nedges:\n") else {
            continue;
        };
        let mut relationship: Option<String> = None;
        let mut inbound = false;
        let mut declared = Vec::new();
        for line in text[start + 1..].lines().skip(1) {
            if line.starts_with("  - ") {
                if let Some(name) = relationship.take() {
                    if !inbound {
                        declared.push(name);
                    }
                }
                inbound = false;
            }
            let key = line.trim_start();
            if let Some(value) = key
                .strip_prefix("- relationship:")
                .or_else(|| key.strip_prefix("relationship:"))
            {
                relationship = Some(value.trim().to_string());
            } else if key.starts_with("direction:") && key.contains("in") {
                inbound = true;
            }
        }
        if let Some(name) = relationship {
            if !inbound {
                declared.push(name);
            }
        }
        out.insert(class.to_string(), declared);
    }
    out
}

fn catalog_audit(root: &Path) -> String {
    let corpus: Vec<String> = corpus_nodes(root)
        .into_iter()
        .map(|(_, _, _, text)| text)
        .collect();
    let mut entries: Vec<(String, String, usize)> = Vec::new();
    if let Ok(files) = fs::read_dir(root.join(".yidam/catalog")) {
        for file in files.flatten() {
            let path = file.path();
            let slug = path
                .file_stem()
                .unwrap_or_default()
                .to_string_lossy()
                .into_owned();
            if path.extension().is_none_or(|e| e != "md") || slug == "README" {
                continue;
            }
            let text = fs::read_to_string(&path).unwrap_or_default();
            let title = text
                .lines()
                .find_map(|line| line.strip_prefix("# "))
                .unwrap_or(&slug)
                .to_string();
            let needle = format!("{slug}.md");
            let cited = corpus.iter().filter(|node| node.contains(&needle)).count();
            entries.push((slug, title, cited));
        }
    }
    entries.sort();

    // Sources whose bytes are pinned by digest, which is a stronger claim than being catalogued.
    let manifest = read(root, crate::cache::MANIFEST);
    let pinned = crate::cache::parse_manifest(&manifest).len();

    let mut out = String::from("| Entry | Title | Cited by |\n|---|---|--:|\n");
    for (slug, title, cited) in &entries {
        out.push_str(&format!("| [`{slug}`]({slug}.md) | {title} | {cited} |\n"));
    }
    let uncited = entries.iter().filter(|(_, _, cited)| *cited == 0).count();
    out.push_str(&format!(
        "\n{} entries, {uncited} not yet cited by any corpus node. {pinned} retrieved files are \
         pinned by SHA-256 in [`crates/connect/source-digests.txt`](../../crates/connect/source-digests.txt).\n",
        entries.len()
    ));
    out
}

/// The test attribute, counted only where it stands alone on a line.
///
/// It used to be counted as a substring anywhere in a `.rs` file, which is a close enough proxy
/// for `cargo test` right up until the counter's own source discusses what it counts. Three of
/// `connect`'s reported tests were this comment, the line below it, and the table heading — and
/// the fourth arrived the moment a doc comment was written above this function. A counter that
/// counts its own description is the same defect as a status that is a string literal: it cannot
/// be wrong in a way that shows.
fn is_test_attribute(line: &str) -> bool {
    line.trim() == "#[test]"
}

/// Every crate in the workspace, as `(name, description, test count)`.
fn crate_rows(root: &Path) -> Vec<(String, String, usize)> {
    let mut rows: Vec<(String, String, usize)> = Vec::new();
    if let Ok(dirs) = fs::read_dir(root.join("crates")) {
        for dir in dirs.flatten() {
            let manifest = dir.path().join("Cargo.toml");
            if !manifest.exists() {
                continue;
            }
            let name = dir.file_name().to_string_lossy().into_owned();
            let text = fs::read_to_string(&manifest).unwrap_or_default();
            let description = toml_field(&text, "description").unwrap_or_default();
            // A close enough proxy for `cargo test`, and unlike that it needs no build.
            let mut tests = 0;
            for entry in walk(&dir.path()) {
                if entry.extension().is_some_and(|e| e == "rs") {
                    tests += fs::read_to_string(&entry)
                        .unwrap_or_default()
                        .lines()
                        .filter(|line| is_test_attribute(line))
                        .count();
                }
            }
            rows.push((name, description, tests));
        }
    }
    rows.sort();
    rows
}

fn crates_index(root: &Path) -> String {
    let rows = crate_rows(root);
    let total: usize = rows.iter().map(|(_, _, tests)| tests).sum();
    let mut out = String::from("| Crate | Description | `#[test]` fns |\n|---|---|--:|\n");
    for (name, description, tests) in &rows {
        out.push_str(&format!(
            "| [`{name}`]({name}/) | {description} | {tests} |\n"
        ));
    }
    out.push_str(&format!(
        "\n{} crates, {total} test functions, no crates.io dependencies. `cargo test` reports a \
         different total: it adds doc-tests and counts each integration binary separately.\n",
        rows.len()
    ));
    out
}

/// Every file under `dir`, recursively, skipping build output.
fn walk(dir: &Path) -> Vec<std::path::PathBuf> {
    let mut out = Vec::new();
    let Ok(entries) = fs::read_dir(dir) else {
        return out;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            if path.file_name().is_some_and(|name| name == "target") {
                continue;
            }
            out.extend(walk(&path));
        } else {
            out.push(path);
        }
    }
    out
}

fn markdown_index(root: &Path, directory: &str, heading: &str) -> String {
    let mut rows: Vec<(String, String)> = Vec::new();
    if let Ok(files) = fs::read_dir(root.join(directory)) {
        for file in files.flatten() {
            let path = file.path();
            let slug = path
                .file_stem()
                .unwrap_or_default()
                .to_string_lossy()
                .into_owned();
            if path.extension().is_none_or(|e| e != "md") || slug == "README" {
                continue;
            }
            let text = fs::read_to_string(&path).unwrap_or_default();
            let description = field(&text, "description")
                .or_else(|| {
                    text.lines()
                        .find_map(|line| line.strip_prefix("**Computes.** "))
                        .map(str::to_string)
                })
                .unwrap_or_default();
            rows.push((slug, description));
        }
    }
    rows.sort();
    let mut out = format!("| {heading} | Description |\n|---|---|\n");
    for (slug, description) in &rows {
        out.push_str(&format!("| [`{slug}`]({slug}.md) | {description} |\n"));
    }
    out
}

/// The root README's inventory of what is in the repository.
///
/// The last table anywhere here that was written by hand, and by the time it was read back every
/// number in it had drifted: 76 nodes against 98, 13 classes against 17, 27 catalog records
/// against 34, 617 test functions against 659. That is the identical failure the connector table
/// and the crate test counts had, and it outlived both fixes for a structural reason — the guard
/// reads markers, and this document had none. Adding one is the whole repair.
///
/// The prose in the right-hand column is qualitative and stays a literal. The counts are not, and
/// none of them is.
fn repository_overview(root: &Path) -> String {
    let nodes = corpus_nodes(root);
    let classes = nodes
        .iter()
        .map(|(class, _, _, _)| class)
        .collect::<std::collections::BTreeSet<_>>()
        .len();
    let crates = crate_rows(root);
    let tests: usize = crates.iter().map(|(_, _, tests)| tests).sum();

    format!(
        "| Directory | What it holds |\n|---|---|\n\
         | [`.yidam/corpus/`](.yidam/corpus/) | The knowledge graph: {} nodes across {classes} \
         classes — regimes, components, parameters, litigation, doctrine, exemplar districts |\n\
         | [`.yidam/catalog/`](.yidam/catalog/) | {} source records. Every numeric claim in the \
         corpus should reach one in a single hop |\n\
         | [`.yidam/decisions/`](.yidam/decisions/) | {} records of why the repository is shaped \
         the way it is, including the ones that turned out wrong |\n\
         | [`crates/`](crates/) | The domain computer: {} Rust crates, {tests} test functions, no \
         crates.io dependencies |\n\
         | [`web/`](web/) | The site — a page per district, statewide views, and a scenario \
         runner, all static |\n\
         | [`agents/`](agents/), [`.yidam/skills/`](.yidam/skills/) | {} traversals and {} \
         procedures this domain keeps needing |\n",
        nodes.len(),
        records(root, ".yidam/catalog", "md"),
        records(root, ".yidam/decisions", "yml"),
        crates.len(),
        records(root, "agents", "md"),
        records(root, ".yidam/skills", "md"),
    )
}

/// How much is retrievable, in one sentence, for a reader who will not open the registry.
///
/// The two blocked connectors used to be *described* here rather than named, in prose that would
/// have gone on reading correctly if a third were blocked and silently wrongly if one were freed.
/// They are named from the registry now, so the sentence cannot outlive the fact.
///
/// # Not-wired is not the same as blocked, and this used to say it was
///
/// The first version took every connector that is not `Wired` and called it "blocked on something
/// this repository cannot build". That was true while the only two were `dew-payment-reports`
/// behind an authenticated portal and `ofcc-projects` behind a browser check. It stopped being
/// true the moment a connector was left unwired **on purpose**: `ohio-bills` is `Retrievable`
/// because turning a bill into provisions is reading rather than extraction, and reporting it as
/// blocked would claim an obstacle that does not exist and imply somebody should go and remove it.
///
/// So the two are counted apart, on the status the registry already carries.
fn retrieval_status(root: &Path) -> String {
    use crate::registry::{Status, CONNECTORS};

    let wired = CONNECTORS.iter().filter(|c| c.status.is_wired()).count();
    let named = |keys: Vec<&'static str>| {
        keys.iter()
            .map(|k| format!("`{k}`"))
            .collect::<Vec<_>>()
            .join(" and ")
    };
    let blocked: Vec<&'static str> = CONNECTORS
        .iter()
        .filter(|c| matches!(c.status, Status::Declared { .. }))
        .map(|c| c.key)
        .collect();
    let unwired: Vec<&'static str> = CONNECTORS
        .iter()
        .filter(|c| matches!(c.status, Status::Retrievable | Status::Parsed))
        .map(|c| c.key)
        .collect();
    let pinned = crate::cache::parse_manifest(&read(root, crate::cache::MANIFEST)).len();

    let mut out = format!(
        "{} connectors, {wired} wired, and {pinned} published files pinned by SHA-256. The {} \
         that {} not — {} — {} blocked on something this repository cannot build, and {} which, \
         in a field a test checks: \
         [the table](crates/connect/README.md) is generated from the registry rather than \
         written by hand.",
        CONNECTORS.len(),
        blocked.len(),
        if blocked.len() == 1 { "is" } else { "are" },
        named(blocked.clone()),
        if blocked.len() == 1 { "is" } else { "are" },
        if blocked.len() == 1 {
            "says"
        } else {
            "each says"
        },
    );
    if !unwired.is_empty() {
        out.push_str(&format!(
            " {} {} unwired by choice rather than by obstacle — a fetchable source whose \
             extraction step is judgement rather than parsing.",
            named(unwired.clone()),
            if unwired.len() == 1 { "is" } else { "are" },
        ));
    }
    out.push('\n');
    out
}

/// The four claim tags and their counts, without the field breakdown the corpus README carries.
///
/// Shares its counter with [`claim_audit`] rather than repeating the loop, because two counts of
/// the same thing in two documents is how the corpus came to report four different totals at once.
fn claim_totals(root: &Path) -> String {
    let totals = claim_tag_totals(root);
    format!(
        "{} `[verified]`, {} `[inference]`, {} `[open]`, {} `[unentered]`.\n",
        totals["[verified]"], totals["[inference]"], totals["[open]"], totals["[unentered]"]
    )
}

/// The connector table, from [`crate::registry::CONNECTORS`] rather than from memory.
///
/// This block exists because the hand-written table it replaced said nine connectors, three of
/// them wired, at a point when the registry held thirteen and eleven. It had gone stale in every
/// direction at once: `census-f33` was described as having no parser while feeding a 10,739-row
/// panel, and `lsc-budget`, `ohio-laws` and `ohio-courts` were all listed `declared` after the
/// phases that wired them. Nothing caught it, because the guard that keeps generated blocks
/// current does not reach prose, and this was prose.
///
/// Every cell is a field. Nothing here is parsed out of a `note`: that is the mistake the
/// DeRolph titles made, where rewording a comment silently rewrote committed data. The notes stay
/// in the registry and the long form stays in `sources/`, both linked, neither transcribed.
fn connector_registry(root: &Path) -> String {
    use crate::registry::{Status, CONNECTORS};

    let mut out = String::from("| Connector | Status | Sources | Feeds |\n|---|---|--:|---|\n");
    let mut undocumented = Vec::new();
    for connector in CONNECTORS {
        let key = connector.key;
        // Linked only when the long form is actually there. Ten of the thirteen have one — the
        // three added after the original stubs never did — and a link to a file that does not
        // exist would be a worse claim than no link.
        let long_form = format!("crates/connect/sources/{key}.md");
        let name = if root.join(&long_form).exists() {
            format!("[`{key}`](sources/{key}.md)")
        } else {
            undocumented.push(key);
            format!("`{key}`")
        };
        let status = match connector.status {
            Status::Wired {
                still_blocked: None,
            } => "**wired**".to_string(),
            Status::Wired { .. } => "**wired**, in part".to_string(),
            other => other.label().to_string(),
        };
        out.push_str(&format!(
            "| {name} | {status} | {} | {} |\n",
            connector.sources.len(),
            connector.feeds.join(", ")
        ));
    }

    let wired = CONNECTORS.iter().filter(|c| c.status.is_wired()).count();
    let partial = CONNECTORS
        .iter()
        .filter(|c| {
            matches!(
                c.status,
                Status::Wired {
                    still_blocked: Some(_)
                }
            )
        })
        .count();
    let sources: usize = CONNECTORS.iter().map(|c| c.sources.len()).sum();
    out.push_str(&format!(
        "\n{} connectors, {sources} sources between them. {wired} are wired and {} are not; \
         {partial} of the wired ones reach only part of what they feed, and say so below.\n",
        CONNECTORS.len(),
        CONNECTORS.len() - wired,
    ));

    // Verbatim, because these strings are the thing a test checks for existence and length. A
    // summary of a blocker is how the last four stale ones survived being read.
    out.push_str("\n**What is blocked, in the registry's own words.**\n\n");
    for connector in CONNECTORS {
        let (kind, reason) = match connector.status {
            Status::Declared { blocked_on } => ("blocked on", blocked_on),
            Status::Wired {
                still_blocked: Some(reason),
            } => ("still blocked on", reason),
            _ => continue,
        };
        out.push_str(&format!("- `{}` — {kind}: {reason}\n", connector.key));
    }

    if !undocumented.is_empty() {
        out.push_str(&format!(
            "\n{} of them have no long form in [`sources/`](sources/): {}. Those are the \
             connectors added after the original nine stubs, whose prose was never written — the \
             decision record is the only account of why each exists.\n",
            undocumented.len(),
            undocumented
                .iter()
                .map(|key| format!("`{key}`"))
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }
    out
}

/// Every claim tag in the corpus, counted, and the unresolved ones by the field they sit in.
///
/// The audit in [`decisions/the-open-item-audit.yml`](../../../.yidam/decisions/) enumerated 152
/// `[open]` claims by hand and its most useful output was the distribution, not the resolutions:
/// it showed that four of the largest clusters were not open questions at all but metadata nobody
/// had typed in. A hand count answers that once. This answers it every time, which matters
/// because the number went to 164 in the phases after the audit and nothing said so.
///
/// Field attribution is by position rather than by parsing YAML: a tag belongs to the last
/// top-level key seen, or to the last two-space key if that top-level key was `properties`. That
/// is exactly the granularity the audit reported in, and it needs no parser in a crate that has
/// deliberately avoided acquiring one.
/// The four tags a corpus claim can carry.
const TAGS: [&str; 4] = ["[verified]", "[inference]", "[open]", "[unentered]"];

/// Count one tag in a body of text, in both the forms the corpus writes it.
///
/// # The 83 claims this used to miss
///
/// A tag may carry its reason — `[verified — the corpus holds this act]` — and 83 of them do.
/// Matching the literal `[verified]` sees none of those, so the generated blocks reported 559
/// verified claims against 642 actually written, understating what the corpus can support by 13%.
///
/// That is the third counter in this crate to count something other than what its name says: the
/// connector table claimed nine connectors against thirteen, and the test counter counted its own
/// description. The shape is always the same — a substring match standing in for a structure — and
/// the fix is always to look at what follows.
///
/// A tag ends at `]` or continues into ` — reason`. Requiring one of those after the name is what
/// keeps `[open]` from also matching a hypothetical `[opening]`.
fn count_tag(text: &str, tag: &str) -> usize {
    let name = tag.trim_start_matches('[').trim_end_matches(']');
    let open = format!("[{name}");
    text.match_indices(&open)
        .filter(|(index, _)| {
            let rest = &text[index + open.len()..];
            rest.starts_with(']') || rest.starts_with(' ')
        })
        .count()
}

/// Every claim tag in the corpus, counted, with no attention to where each one sits.
///
/// The one counter. [`claim_audit`] renders it with a field breakdown and [`claim_totals`]
/// renders it bare; neither counts for itself, so the root README and the corpus README cannot
/// disagree about how much this repository knows.
fn claim_tag_totals(root: &Path) -> BTreeMap<&'static str, usize> {
    let mut totals: BTreeMap<&str, usize> = TAGS.iter().map(|tag| (*tag, 0)).collect();
    for (_, _, _, text) in corpus_nodes(root) {
        for tag in TAGS {
            *totals.get_mut(tag).expect("tag is in the map") += count_tag(&text, tag);
        }
    }
    totals
}

fn claim_audit(root: &Path) -> String {
    let nodes = corpus_nodes(root);
    let totals = claim_tag_totals(root);
    // Only the unresolved tags get a field breakdown; `[verified]` by field says nothing.
    let mut unresolved: BTreeMap<(String, &str), usize> = BTreeMap::new();

    for (_, _, _, text) in &nodes {
        let mut top = String::new();
        let mut field = String::new();
        for line in text.lines() {
            if let Some(key) = line.split_once(':').map(|(key, _)| key) {
                if !key.is_empty() && !key.starts_with(char::is_whitespace) && !key.contains(' ') {
                    top = key.to_string();
                    field.clone_from(&top);
                } else if top == "properties"
                    && line.starts_with("  ")
                    && !line.starts_with("   ")
                    && !key.trim().contains(' ')
                {
                    field = key.trim().to_string();
                }
            }
            for tag in TAGS {
                let hits = count_tag(line, tag);
                if hits == 0 || tag == "[verified]" || tag == "[inference]" {
                    continue;
                }
                *unresolved.entry((field.clone(), tag)).or_default() += hits;
            }
        }
    }

    let mut out = String::from("| Tag | Count | What it records |\n|---|--:|---|\n");
    for (tag, meaning) in [
        ("[verified]", "supported by a committed primary source"),
        ("[inference]", "drawn from verified facts, not witnessed"),
        (
            "[open]",
            "a live question — unknown, contested, or being worked",
        ),
        ("[unentered]", "a knowable value nobody has typed in yet"),
    ] {
        out.push_str(&format!("| `{tag}` | {} | {meaning} |\n", totals[tag]));
    }

    let open = totals["[open]"];
    let unentered = totals["[unentered]"];
    // The zero case is the ordinary one now and needs its own sentence: "overstated it by 0%" is
    // not a fact about anything. `[unentered]` was migrated out of the prose entirely — an empty
    // field is carried as `unfilled:` structure on the node, not as a fourth mark in somebody's
    // sentence — so a nonzero count here means a node has started writing the retired mark again.
    if unentered == 0 {
        out.push_str(&format!(
            "\n{open} unresolved marks, and every one of them is a live question. The fourth mark \
             is gone from the prose: a field nobody has filled in is carried as `unfilled:` \
             structure on the node it belongs to, which is what `[unentered]` used to say inline \
             on an axis it did not belong to.\n"
        ));
    } else {
        out.push_str(&format!(
            "\n{} unresolved marks in total, {open} of them live questions and {unentered} of \
             them empty fields — and that second number should be zero, because the mark was \
             migrated to `unfilled:` structure. A node has started writing it again.\n",
            open + unentered
        ));
    }

    out.push_str("\n| Field | `[open]` | `[unentered]` |\n|---|--:|--:|\n");
    let fields: std::collections::BTreeSet<&String> =
        unresolved.keys().map(|(field, _)| field).collect();
    let mut rows: Vec<(usize, usize, &String)> = fields
        .into_iter()
        .map(|field| {
            (
                unresolved
                    .get(&(field.clone(), "[open]"))
                    .copied()
                    .unwrap_or(0),
                unresolved
                    .get(&(field.clone(), "[unentered]"))
                    .copied()
                    .unwrap_or(0),
                field,
            )
        })
        .collect();
    rows.sort_by(|a, b| (b.0 + b.1, b.2).cmp(&(a.0 + a.1, a.2)));
    for (open, unentered, field) in rows {
        out.push_str(&format!("| `{field}` | {open} | {unentered} |\n"));
    }

    let (nodes_with, entries) = revisions(&nodes);
    out.push_str(&format!(
        "\n**{entries} recorded withdrawals across {nodes_with} nodes.** A claim the corpus \
         published and no longer stands behind is kept in a `revisions:` block rather than \
         edited out, with the test or source that settled it — see \
         [`the-four-genres-of-a-description`](../decisions/the-four-genres-of-a-description.yml). \
         Counted here for the same reason the tags above are: how often this corpus has corrected \
         itself is a fact about it, and one nobody would think to update by hand.\n"
    ));
    out
}

/// Withdrawals in the corpus, as `(nodes carrying any, entries in total)`.
///
/// Counted by scanning for the `revisions:` key and the `- was:` entries under it, in the same
/// positional style as [`claim_audit`] and for the same reason: this crate has deliberately not
/// acquired a YAML parser, and the shape being counted is one line deep.
///
/// The entry count is what matters and the node count is what makes it readable — three
/// withdrawals on one node and three across three are different facts about a corpus, and a
/// single number cannot tell them apart.
fn revisions(nodes: &[(String, String, String, String)]) -> (usize, usize) {
    let mut nodes_with = 0;
    let mut entries = 0;
    for (_, _, _, text) in nodes {
        let mut inside = false;
        let mut found = 0;
        for line in text.lines() {
            if line.starts_with("revisions:") {
                inside = true;
            } else if inside && !line.starts_with(char::is_whitespace) && !line.trim().is_empty() {
                // A new top-level key. `properties:` follows `revisions:` in every node that has
                // both, and without this the `- target:` entries under `links:` would be counted.
                inside = false;
            } else if inside && line.trim_start().starts_with("- was:") {
                found += 1;
            }
        }
        if found > 0 {
            nodes_with += 1;
            entries += found;
        }
    }
    (nodes_with, entries)
}

/// Why there is no semantic index, and how much corpus there is to not index.
///
/// The node count used to be the literal `58` returned from this function, which is how it came
/// to sit nine lines below a generated block reading 74 and survive every CI run: regenerating a
/// constant produces no diff. A status block that cannot go stale is one that measures something.
fn index_status(root: &Path) -> String {
    format!(
        "No semantic index is built. The corpus is {} nodes and fits in context; an index is \
         added when direct retrieval stops working, which has not happened.\n",
        corpus_nodes(root).len()
    )
}

fn bundle_status(root: &Path) -> String {
    let feed = read(root, "web/public/data/bundle.json");
    let version = feed
        .lines()
        .find_map(|line| line.trim().strip_prefix("\"contract_version\": "))
        .map(|value| value.trim_matches(|c| c == '"' || c == ',').to_string())
        .unwrap_or_else(|| "unknown".into());
    // Counted by a field only a full district record carries. `"irn": ` was the discriminator
    // until the feed gained House districts, whose member rows carry an IRN too: the count went
    // from 609 to 1,694 without anything else changing. A key that is unique today is a
    // discriminator only until the next block is added, and "count the occurrences of a common
    // field name" has now failed here twice for the same reason.
    //
    // `"adm_history": ` is a district's three-year enrolment series. It is a poor thing to search
    // for and a good thing to count: nothing else in the feed has one, and if something ever does
    // it will be another per-district block rather than a nested row.
    let districts = feed.matches("\"adm_history\": ").count();
    // Each kind is counted by a field only that kind carries, rather than by subtracting one
    // count from another. `label` was the discriminator once and stopped being unique the moment
    // the deflator acquired one — a subtraction is only as stable as every other user of the
    // field it subtracts from.
    let checkpoints = feed.matches("\"cost\": ").count();
    let forecasts = feed.matches("\"low\": ").count();
    format!(
        "| Field | Value |\n|---|---|\n\
         | Contract version | `{version}` |\n\
         | Districts in the feed | {districts} |\n\
         | Reference checkpoints | {checkpoints} |\n\
         | Reference forecasts | {forecasts} |\n\
         | Size | {} KB |\n\
         | Deployment target | Cloudflare Pages, static, with a CSP in `web/public/_headers` |\n\n\
         Regenerate with `cargo run --manifest-path crates/Cargo.toml -p bundle > \
         web/public/data/bundle.json`. CI fails if the committed feed and a fresh one differ.\n",
        feed.len() / 1024
    )
}

/// Fill one block, given the command in its marker.
fn generate(command: &str, root: &Path) -> Option<String> {
    Some(match command {
        "yidam corpus-index" => corpus_index(root),
        "yidam catalog-audit" => catalog_audit(root),
        "yidam crates-index" => crates_index(root),
        "yidam skills-index" => markdown_index(root, ".yidam/skills", "Skill"),
        "yidam agents-index" => markdown_index(root, "agents", "Agent"),
        "yidam bundle-status" => bundle_status(root),
        "yidam connector-registry" => connector_registry(root),
        "yidam claim-audit" => claim_audit(root),
        "yidam edge-vocabulary" => edge_vocabulary(root),
        "yidam repository-overview" => repository_overview(root),
        "yidam retrieval-status" => retrieval_status(root),
        "yidam claim-totals" => claim_totals(root),
        // Describes a system this repository does not have. Saying so beats an empty block and
        // beats a fabricated status — but the sentence still has to count what it counts.
        "yidam index-status" => index_status(root),
        _ => return None,
    })
}

/// Regenerate every block in every document. Returns the files that changed.
///
/// # Errors
///
/// Returns [`IndexError::UnknownBlock`] if a document names a generator that does not exist —
/// a new block is then a deliberate act rather than something that silently stays empty.
pub fn regenerate(root: &Path) -> Result<Vec<String>, IndexError> {
    rewrite(root, true)
}

/// Which documents carry a stale block, without touching any of them.
///
/// The form a gate wants. Regenerating and then asking `git diff --quiet` — which is what
/// `.github/workflows/ci.yml` does — cannot distinguish a stale block from an unrelated
/// uncommitted edit in the same file, so it reports a false failure against any dirty working
/// tree. CI never met that case because CI checks out clean; a person running the same check
/// locally meets it immediately, and a gate that cries wolf on every edit is a gate that gets
/// ignored.
///
/// # Errors
///
/// As [`regenerate`].
pub fn stale(root: &Path) -> Result<Vec<String>, IndexError> {
    rewrite(root, false)
}

/// Render every block; write when asked. Returns the documents whose content would change.
fn rewrite(root: &Path, write: bool) -> Result<Vec<String>, IndexError> {
    let mut changed = Vec::new();
    for document in DOCUMENTS {
        let path = root.join(document);
        let Ok(original) = fs::read_to_string(&path) else {
            continue;
        };
        let mut out = String::with_capacity(original.len());
        let mut rest = original.as_str();

        while let Some(start) = rest.find("<!-- REGEN: ") {
            let (before, from_marker) = rest.split_at(start);
            out.push_str(before);
            let Some(marker_end) = from_marker.find("-->") else {
                break;
            };
            let Some(close) = from_marker.find("<!-- /REGEN -->") else {
                break;
            };
            let command = from_marker["<!-- REGEN: ".len()..]
                .lines()
                .next()
                .unwrap_or_default()
                .trim()
                .to_string();
            let body = generate(&command, root).ok_or_else(|| IndexError::UnknownBlock {
                command: command.clone(),
                file: (*document).to_string(),
            })?;
            out.push_str(&from_marker[..marker_end + 3]);
            out.push('\n');
            out.push_str(&body);
            rest = &from_marker[close..];
        }
        out.push_str(rest);

        if out != original {
            if write {
                fs::write(&path, out)?;
            }
            changed.push((*document).to_string());
        }
    }
    Ok(changed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cache::repository_root;

    #[test]
    fn every_block_in_every_document_has_a_generator() {
        // The check that keeps a heading from promising content nothing produces. Adding a new
        // REGEN block without a generator fails here rather than shipping an empty section.
        let root = repository_root();
        for document in DOCUMENTS {
            let text = std::fs::read_to_string(root.join(document)).unwrap_or_default();
            for marker in text.split("<!-- REGEN: ").skip(1) {
                let command = marker.lines().next().unwrap_or_default().trim();
                assert!(
                    generate(command, &root).is_some(),
                    "{document}: no generator for {command:?}"
                );
            }
        }
    }

    #[test]
    fn no_document_still_tells_a_reader_to_run_a_binary_that_does_not_exist() {
        let root = repository_root();
        for document in DOCUMENTS {
            let text = std::fs::read_to_string(root.join(document)).unwrap_or_default();
            assert!(
                !text.contains("to populate._"),
                "{document} still carries an unpopulated block"
            );
        }
    }

    #[test]
    fn the_corpus_index_counts_links_in_both_directions() {
        let index = corpus_index(&repository_root());
        assert!(index.contains("| Node | Class | Label | Out | In |"));
        assert!(index.contains("nodes across"));
        assert!(index.contains("with nothing pointing at them"));
        // The sentence used to call every one of those a gap. Two are `draft-legislation`, where
        // nothing pointing at a node is the class working — so it names the check that knows the
        // difference instead of asserting the wrong half of it.
        assert!(index.contains("reachability.spec.ts"));
    }

    #[test]
    fn the_crates_index_finds_every_crate_and_some_tests() {
        let index = crates_index(&repository_root());
        for crate_name in ["spreadsheet", "connect", "project", "bundle"] {
            assert!(index.contains(&format!("[`{crate_name}`]")), "{crate_name}");
        }
        assert!(index.contains("no crates.io dependencies"));
    }

    #[test]
    fn the_bundle_status_reads_the_feed_rather_than_describing_it() {
        let status = bundle_status(&repository_root());
        // Against the version *in the feed*, which is the property the name claims: the block
        // reports what it read rather than what someone typed. A literal here turns every contract
        // bump into an unrelated failure and teaches whoever hits it to edit the assertion.
        let feed = read(&repository_root(), "web/public/data/bundle.json");
        let declared = feed
            .split("\"contract_version\": \"")
            .nth(1)
            .and_then(|rest| rest.split('"').next())
            .expect("the feed declares a contract version");
        assert!(status.contains(&format!("`{declared}`")), "{status}");
        assert!(
            status.contains("| Districts in the feed | 609 |"),
            "{status}"
        );
    }

    #[test]
    fn the_two_kinds_of_checkpoint_are_counted_apart() {
        // They gate different things — the scenario builder and the projection band — and the
        // page reports them as two numbers. One combined count would match neither.
        let status = bundle_status(&repository_root());
        assert!(status.contains("| Reference checkpoints | 8 |"), "{status}");
        assert!(status.contains("| Reference forecasts | 4 |"), "{status}");
    }

    #[test]
    fn a_status_for_a_system_that_does_not_exist_says_so() {
        let text = generate("yidam index-status", &repository_root()).unwrap();
        assert!(text.contains("No semantic index is built"));
    }

    #[test]
    fn the_index_status_counts_the_corpus_rather_than_remembering_it() {
        // The defect: this sentence was a literal, so it regenerated to itself and rendered nine
        // lines below a computed block that disagreed with it. Asserting against the same node
        // walk the corpus index uses is the only version of this test that can fail when it
        // should — a literal here would restore exactly what went wrong.
        let root = repository_root();
        let nodes = corpus_nodes(&root).len();
        assert!(
            index_status(&root).contains(&format!("is {nodes} nodes")),
            "the status and the corpus disagree"
        );
        assert!(
            corpus_index(&root).contains(&format!("\n{nodes} nodes across")),
            "the two blocks in .yidam/corpus/README.md count differently"
        );
    }

    #[test]
    fn the_claim_audit_counts_every_tag_the_corpus_actually_carries() {
        let root = repository_root();
        let audit = claim_audit(&root);
        let nodes = corpus_nodes(&root);
        for tag in TAGS {
            let actual: usize = nodes
                .iter()
                .map(|(_, _, _, text)| count_tag(text, tag))
                .sum();
            assert!(
                audit.contains(&format!("| `{tag}` | {actual} |")),
                "{tag} is reported as something other than {actual}"
            );
        }
    }

    #[test]
    fn the_edge_vocabulary_counts_edges_between_nodes_and_nothing_else() {
        // Built from lines rather than written as one escaped literal: rustfmt rewraps a long
        // string with continuations and folds the source indentation into the value, which
        // silently changes what is being tested.
        let node = [
            "class: legislation",
            "links:",
            "  - target: ../legislation.ont.yml",
            "    relationship: instance-of",
            "  - target: hb-110-2021.yml",
            "    relationship: amends",
            "  - target: ../fiscal-period/fy2026-27.yml",
            "    relationship: appropriates-for",
            "    note: >-",
            "      A body indented six spaces, mentioning target: and relationship: in prose.",
            "  - target: ../../catalog/lsc-hb96-analysis.md",
            "    relationship: sourced-from",
            "",
        ]
        .join("\n");

        let edges = node_edges(&node);
        let pairs: Vec<(&str, &str)> = edges
            .iter()
            .map(|(target, relationship)| (target.as_str(), relationship.as_str()))
            .collect();
        assert_eq!(
            pairs,
            vec![
                ("../legislation.ont.yml", "instance-of"),
                ("hb-110-2021.yml", "amends"),
                ("../fiscal-period/fy2026-27.yml", "appropriates-for"),
                ("../../catalog/lsc-hb96-analysis.md", "sourced-from"),
            ],
            "every entry is read once, and the note body is not mistaken for one"
        );

        let counted = edges
            .iter()
            .filter(|(target, _)| target.ends_with(".yml") && !target.ends_with(".ont.yml"))
            .count();
        assert_eq!(counted, 2, "the ontology and the catalog leave the corpus");
    }

    /// An `in` edge is written on the other node, so it is not something this class declares.
    ///
    /// Counting it would inflate the declared total and understate how far the corpus's actual
    /// vocabulary runs past its ontologies — which is the whole quantity this block exists to
    /// report, and the one that was wrong in seventeen hand-written copies.
    #[test]
    fn only_out_direction_relationships_count_as_declared() {
        let declared = ontology_edges(&repository_root());
        let legislation = declared.get("legislation").expect("the class exists");
        assert!(legislation.contains(&"appropriates-for".to_string()));
        assert!(
            !legislation.contains(&"enacted-by".to_string()),
            "enacted-by is declared `direction: in` and belongs to actor, not here"
        );
    }

    #[test]
    fn the_edge_vocabulary_block_agrees_with_a_direct_count() {
        let root = repository_root();
        let total: usize = corpus_nodes(&root)
            .iter()
            .flat_map(|(_, _, _, text)| node_edges(text))
            .filter(|(target, _)| target.ends_with(".yml") && !target.ends_with(".ont.yml"))
            .count();
        assert!(
            edge_vocabulary(&root).contains(&format!("| edges between nodes | {total} |")),
            "the generated block and a direct count disagree"
        );
    }

    #[test]
    fn a_tag_carrying_its_reason_is_still_counted() {
        // The defect: 83 claims were written `[verified — because]` and the counter matched the
        // literal `[verified]`, so none of them existed as far as the generated blocks were
        // concerned. Both forms are one claim each.
        assert_eq!(count_tag("a [verified] b", "[verified]"), 1);
        assert_eq!(
            count_tag("a [verified — the corpus holds it] b", "[verified]"),
            1
        );
        assert_eq!(
            count_tag("[open] and [open — pending a source]", "[open]"),
            2
        );
        // And the check that the reason form does not swallow a longer word.
        assert_eq!(count_tag("[opening remarks]", "[open]"), 0);
        assert_eq!(count_tag("[verifiedish]", "[verified]"), 0);
    }

    #[test]
    fn the_two_kinds_of_unresolved_mark_are_reported_apart() {
        // The whole point of the notation. A single total is what the audit had, and it read as
        // 152 open questions when a seventh of them were empty fields nobody had typed into.
        let audit = claim_audit(&repository_root());
        assert!(audit.contains("| `[open]` |"), "{audit}");
        assert!(audit.contains("| `[unentered]` |"), "{audit}");
        assert!(
            audit.contains("| Field | `[open]` | `[unentered]` |"),
            "{audit}"
        );
    }

    #[test]
    fn a_tag_is_attributed_to_the_field_it_sits_in() {
        // Attribution is positional, so the case that would break it silently is a property whose
        // value runs over several lines: the tag lands on a continuation line, not on the `key:`.
        let root = repository_root();
        let audit = claim_audit(&root);
        // The fields this named — `established`, `typology`, `series_path` — carried the fourth
        // mark and were migrated to `unfilled:` structure, so they attribute nothing now and the
        // test failed on its own examples rather than on the behaviour. Replaced with fields that
        // still carry marks in both shapes: `exit` is a one-line property, `statutory_basis` a
        // block scalar, and `description` and `findings` are the top-level prose fields.
        for field in ["exit", "statutory_basis", "description", "findings"] {
            assert!(
                audit.contains(&format!("| `{field}` |")),
                "{field} is not attributed at all"
            );
        }
    }

    #[test]
    fn the_root_readme_counts_the_directories_it_describes() {
        // The block that exists because this table was the last hand-written one, and by the time
        // anyone read it back it was wrong about nodes, classes, catalog records and tests at
        // once. Every assertion here is against the filesystem, so the table cannot drift from it.
        let root = repository_root();
        let overview = repository_overview(&root);
        let nodes = corpus_nodes(&root);
        let classes = nodes
            .iter()
            .map(|(class, _, _, _)| class)
            .collect::<std::collections::BTreeSet<_>>()
            .len();
        assert!(
            overview.contains(&format!("{} nodes across {classes} classes", nodes.len())),
            "{overview}"
        );
        assert!(
            overview.contains(&format!(
                "{} source records",
                records(&root, ".yidam/catalog", "md")
            )),
            "{overview}"
        );
        assert!(
            overview.contains(&format!("{} Rust crates", crate_rows(&root).len())),
            "{overview}"
        );
    }

    #[test]
    fn the_root_readme_and_the_corpus_readme_agree_about_what_is_known() {
        // Two documents reporting the same four counts is exactly the arrangement that let the
        // corpus publish four different totals at once. They share a counter; this is the test
        // that says so, and it fails if either grows its own.
        let root = repository_root();
        let totals = claim_totals(&root);
        let audit = claim_audit(&root);
        for tag in TAGS {
            let count = claim_tag_totals(&root)[tag];
            assert!(
                totals.contains(&format!("{count} `{tag}`")),
                "the root README reports {tag} as something other than {count}"
            );
            assert!(
                audit.contains(&format!("| `{tag}` | {count} |")),
                "the corpus README reports {tag} as something other than {count}"
            );
        }
    }

    #[test]
    fn the_retrieval_sentence_names_the_blocked_connectors_rather_than_describing_them() {
        // It used to describe them — "an authenticated portal, and a publisher that serves
        // interactive maps" — which reads correctly whether or not those are still the two, and
        // would have gone on reading correctly if one were freed.
        let root = repository_root();
        let sentence = retrieval_status(&root);
        for connector in crate::registry::CONNECTORS {
            let named = sentence.contains(&format!("`{}`", connector.key));
            assert_eq!(
                named,
                !connector.status.is_wired(),
                "{} is {}named in the root README's retrieval sentence",
                connector.key,
                if named { "" } else { "not " }
            );
        }
    }

    #[test]
    fn the_test_counter_does_not_count_itself_talking_about_tests() {
        // The defect this function was extracted to fix. Counting the attribute as a substring
        // made `connect` report three tests it did not have — a comment, a string literal and a
        // table heading, all in the counter's own source — and adding a doc comment above the
        // counter added a fourth.
        assert!(is_test_attribute("#[test]"));
        assert!(is_test_attribute("    #[test]"));
        assert!(!is_test_attribute(
            "/// Counting `#[test]` attributes across the crate"
        ));
        assert!(!is_test_attribute("        .matches(\"#[test]\")"));
        assert!(!is_test_attribute(
            "| Crate | Description | `#[test]` fns |"
        ));
    }

    #[test]
    fn the_connector_table_reports_every_connector_and_its_real_status() {
        let table = connector_registry(&repository_root());
        for connector in crate::registry::CONNECTORS {
            assert!(
                table.contains(&format!("`{}`", connector.key)),
                "{} is missing from the table",
                connector.key
            );
        }
        let wired = crate::registry::CONNECTORS
            .iter()
            .filter(|c| c.status.is_wired())
            .count();
        assert!(
            table.contains(&format!("{wired} are wired")),
            "the count and the registry disagree: {table}"
        );
    }

    #[test]
    fn every_blocker_reaches_the_page_verbatim() {
        // A blocker that is summarised is a blocker nobody rechecks. Four of them sat stale for
        // twelve phases behind prose that read plausibly, so the string a test guards for length
        // is the string the reader gets.
        use crate::registry::Status;
        let table = connector_registry(&repository_root());
        for connector in crate::registry::CONNECTORS {
            let reason = match connector.status {
                Status::Declared { blocked_on } => blocked_on,
                Status::Wired {
                    still_blocked: Some(reason),
                } => reason,
                _ => continue,
            };
            assert!(
                table.contains(reason),
                "{}'s blocker is not on the page",
                connector.key
            );
        }
    }

    #[test]
    fn an_unknown_block_is_an_error_rather_than_a_silent_skip() {
        assert!(generate("yidam invented-command", &repository_root()).is_none());
    }

    #[test]
    fn every_edge_is_counted_once_from_each_end() {
        // Out and In are the same relation read in opposite directions, so their totals have to
        // agree. That is worth asserting because they were computed by unrelated code for a long
        // time: Out counted markdown links in the prose and In counted nodes mentioning a slug,
        // and nothing would have told anyone the column headings implied a symmetry the numbers
        // did not have.
        let root = repository_root();
        let nodes = corpus_nodes(&root);
        let slugs: Vec<&str> = nodes.iter().map(|(_, slug, _, _)| slug.as_str()).collect();

        let out: usize = nodes
            .iter()
            .map(|(_, slug, _, text)| outgoing_links(text, &slugs, slug))
            .sum();
        let inbound: usize = slugs
            .iter()
            .map(|slug| {
                let needle = format!("{slug}.yml");
                nodes
                    .iter()
                    .filter(|(_, other, _, text)| other != slug && text.contains(&needle))
                    .count()
            })
            .sum();

        assert_eq!(out, inbound, "the graph's two directions disagree");
        assert!(
            out > 0,
            "no edges found at all, which means the match is broken"
        );
    }

    #[test]
    fn a_structured_link_counts_as_much_as_a_prose_one() {
        // The defect this replaced. A node that declares its edges in `links:` and a node that
        // writes the same edges as sentences have the same out-degree; the old counter saw only
        // the second, so moving a node from prose to structure made its number fall.
        let slugs = ["adequacy", "equity"];
        let structured =
            "links:\n  - target: ../doctrine/adequacy.yml\n    relationship: bears-on\n";
        let prose = "Bears on [adequacy](../doctrine/adequacy.yml).\n";
        assert_eq!(outgoing_links(structured, &slugs, "x"), 1);
        assert_eq!(outgoing_links(prose, &slugs, "x"), 1);

        // And a target named twice is one edge, not two.
        let twice = "[adequacy](../doctrine/adequacy.yml) and again ../doctrine/adequacy.yml";
        assert_eq!(outgoing_links(twice, &slugs, "x"), 1);
    }

    #[test]
    fn checking_reports_what_regenerating_would_change_and_writes_nothing() {
        // The two paths share one renderer, so what this really guards is that `--check` cannot
        // acquire a side effect: the gate runs it against a dirty tree, and a check that writes
        // would quietly stage work nobody asked for.
        let root = repository_root();
        regenerate(&root).expect("generators exist");
        let before: Vec<String> = DOCUMENTS
            .iter()
            .map(|document| std::fs::read_to_string(root.join(document)).unwrap_or_default())
            .collect();

        assert!(
            stale(&root).expect("generators exist").is_empty(),
            "check disagrees with a regeneration that just ran"
        );

        for (document, original) in DOCUMENTS.iter().zip(before) {
            assert_eq!(
                std::fs::read_to_string(root.join(document)).unwrap_or_default(),
                original,
                "{document} was written by a check"
            );
        }
    }

    #[test]
    fn regenerating_twice_changes_nothing_the_second_time() {
        // Idempotence is what makes the CI check meaningful: a diff after regeneration means
        // the documents were stale, not that the generator is unstable.
        let root = repository_root();
        regenerate(&root).expect("generators exist");
        assert!(
            regenerate(&root).expect("generators exist").is_empty(),
            "regeneration is not idempotent"
        );
    }
}
