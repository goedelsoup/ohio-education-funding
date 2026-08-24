//! What a draft is not allowed to do.
//!
//! The [`draft-legislation`](../../../.yidam/corpus/draft-legislation/) class rests on one rule —
//! a cost cannot be stated without the count of provisions it fails to price — and on the fixture
//! and the corpus node saying the same thing. Both are asserted here rather than trusted.
//!
//! The rule is the transposition of `scenario-delta`'s: a total there cannot be constructed
//! without the count of districts it fails to reach. What makes it worth enforcing rather than
//! documenting is that the provisions binding to levers are exactly the provisions that produce a
//! number, so the wrong answer is the one that falls out of doing the obvious thing.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use project::drafts::{draft, drafts, price, Lever};
use project::panel::panel;
use project::policy::{GuaranteeRule, Policy};
use project::report::simulate;

/// The repository root. `CARGO_MANIFEST_DIR` is `<repo>/crates/project`, so the root is two up.
fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .unwrap_or(Path::new("."))
        .to_path_buf()
}

/// Whether a node's text names an authority.
///
/// Case-folded, because `uncodified` is a sentinel written as an ordinary word — it opens a
/// sentence in one node and sits mid-clause in another, and a case-sensitive check would be
/// testing capitalisation rather than whether the provision is documented. Revised Code citations
/// are unaffected either way.
fn names(text: &str, authority: &str) -> bool {
    text.to_lowercase().contains(&authority.to_lowercase())
}

fn node(slug: &str) -> String {
    let path = root()
        .join(".yidam/corpus/draft-legislation")
        .join(format!("{slug}.yml"));
    std::fs::read_to_string(&path)
        .unwrap_or_else(|_| panic!("draft {slug} has no corpus node at {}", path.display()))
}

#[test]
fn every_draft_in_the_fixture_has_a_corpus_node() {
    // The fixture is what runs and the node is what a reader reads. A draft priced by the CLI and
    // absent from the corpus would be a number with no document behind it, which is the shape
    // this repository keeps its knowledge in specifically to avoid.
    for slug in drafts().keys() {
        assert!(
            !node(slug).is_empty(),
            "draft {slug} is in the fixture with an empty corpus node"
        );
    }
}

#[test]
fn the_node_names_every_provision_the_fixture_binds() {
    // The reconciliation, in the direction that matters. The fixture is authoritative — it is what
    // `price` reads — so the failure to catch is the node quietly omitting a provision, which
    // would leave a reader with a shorter bill than the one that was costed.
    //
    // Checked on the parameter slug and the proposed value rather than the title, because a title
    // is prose and will be reworded, while a binding that changes is a different provision.
    for (slug, draft) in drafts() {
        let text = node(&slug);
        for provision in &draft.provisions {
            if !provision.parameter.is_empty() {
                assert!(
                    text.contains(&provision.parameter),
                    "{slug} provision {} binds {} and the node never names it",
                    provision.ordinal,
                    provision.parameter
                );
            }
            assert!(
                names(&text, &provision.authority),
                "{slug} provision {} amends {} and the node never names it",
                provision.ordinal,
                provision.authority
            );
        }
    }
}

#[test]
fn the_node_admits_what_the_fixture_could_not_price() {
    // The prose half of the invariant. A node whose `unpriced:` says "None" while the fixture
    // carries three unlevered provisions is the exact failure the field exists to prevent, and it
    // is invisible to any check that only reads the fixture.
    for (slug, draft) in drafts() {
        let text = node(&slug);
        let unpriced = draft.unpriced();
        if unpriced.is_empty() {
            assert!(
                text.contains("unpriced: |\n    None"),
                "{slug} prices every provision, and its node has to say so in those words"
            );
        } else {
            for provision in unpriced {
                assert!(
                    names(&text, &provision.authority),
                    "{slug} cannot price provision {} and the node does not name it",
                    provision.ordinal
                );
            }
        }
    }
}

#[test]
fn an_unpriced_provision_says_what_it_would_need() {
    // "Not runnable" on its own is a shrug. The difference between a question somebody can work
    // and one nobody has scoped is entirely in this field, and an empty one erases it.
    for (slug, draft) in drafts() {
        for provision in draft.unpriced() {
            assert!(
                provision.note.len() > 40,
                "{slug} provision {} is unpriced with no account of why: {:?}",
                provision.ordinal,
                provision.note
            );
        }
    }
}

#[test]
fn a_priced_provision_binds_a_parameter_node_that_exists() {
    // A `simulation_key` pointing at nothing is worse than none: it reads as provenance.
    for (slug, draft) in drafts() {
        for provision in draft.priced() {
            assert!(
                !provision.parameter.is_empty(),
                "{slug} provision {} moves a lever and names no parameter",
                provision.ordinal
            );
            let path = root()
                .join(".yidam/corpus/parameter")
                .join(format!("{}.yml", provision.parameter));
            assert!(
                path.exists(),
                "{slug} provision {} binds {}, which is not a corpus node",
                provision.ordinal,
                provision.parameter
            );
        }
    }
}

#[test]
fn a_cost_arrives_with_the_count_it_does_not_include() {
    // The type-level version of the rule. `Priced` has no constructor but `price`, and `price`
    // always computes both halves, so there is no path from a draft to a bare dollar figure.
    let districts = panel();
    for draft in drafts().into_values() {
        let priced = price(&draft, &districts);
        assert_eq!(
            priced.provisions(),
            draft.provisions.len(),
            "{}: the priced and unpriced lists have to account for every provision",
            draft.slug
        );
        assert_eq!(
            priced.attribution().len() + priced.unpriced().len(),
            draft.provisions.len(),
            "{}: a provision fell out between the two lists",
            draft.slug
        );
    }
}

#[test]
fn the_single_provision_draft_reproduces_the_run_it_is_linked_to() {
    // `hb-96-with-refreshed-inputs` links to `scenario/fsfp-input-year-refresh`, whose figure is
    // pinned in `scenario-delta` at $220.6M delivered against 356 gainers. If the draft machinery
    // resolves the provision correctly it lands on the same number by a different route.
    let districts = panel();
    let priced = price(&draft("hb-96-with-refreshed-inputs").unwrap(), &districts);
    let cost = priced
        .cost()
        .expect("one provision prices, so there is a figure");

    assert!(
        (cost / 1e6 - 220.6).abs() < 1.0,
        "the refresh draft delivers {:.1}M, and the scenario it cites says 220.6M",
        cost / 1e6
    );
    assert_eq!(priced.effect().gainers(), 356);
    assert!(priced.unpriced().is_empty());
}

#[test]
fn a_one_provision_draft_has_no_residual() {
    // The arithmetic check on `residual`. With a single provision the combined run and the
    // attribution are the same run, so anything but zero means the two paths disagree.
    let districts = panel();
    let priced = price(&draft("hb-96-with-refreshed-inputs").unwrap(), &districts);
    let residual = priced
        .residual()
        .expect("one provision prices, so there is a residual");
    assert!(
        residual.abs() < 1.0,
        "a single-provision draft residual should be zero, got {residual}"
    );
}

#[test]
fn the_parts_do_not_sum_to_the_whole() {
    /*
     * The finding that makes attribution a separate thing from cost.
     *
     * `fund-the-plan-and-retire-the-guarantee` raises base cost and retires half the guarantee.
     * Priced apart they say -$219.0M; run together they say -$143.9M. The difference is $75.1M —
     * 52% of the combined figure — and it is not rounding, it is the guarantee's `max` counted
     * twice. A district the refresh lifts off the floor is not standing on the floor for the
     * phase-out to lower.
     *
     * The assertion is on the magnitude rather than the sign alone, because a residual that
     * quietly collapsed to a rounding error would mean the levers had stopped interacting, and
     * that is a change to the model worth failing on.
     */
    let districts = panel();
    let priced = price(
        &draft("fund-the-plan-and-retire-the-guarantee").unwrap(),
        &districts,
    );

    let combined = priced.cost().expect("two provisions price");
    let residual = priced.residual().expect("two provisions price");
    let apart: f64 = priced.attribution().iter().map(|a| a.cost).sum();

    assert!((combined / 1e6 + 143.9).abs() < 1.0, "combined {combined}");
    assert!((apart / 1e6 + 219.0).abs() < 1.0, "apart {apart}");
    assert!(
        residual / 1e6 > 70.0,
        "the residual is the whole reason attribution is reported apart; got {:.1}M",
        residual / 1e6
    );
    assert!(
        residual / combined.abs() > 0.5,
        "the residual is more than half the combined figure, which is why clauses cannot be \
         priced independently and added"
    );
}

#[test]
fn the_two_priced_provisions_cover_the_state_without_partitioning_it() {
    /*
     * Why the combination touches every district, and why that is a weaker result than it looks.
     *
     * The two *unmoved* sets are disjoint: 253 held on the guarantee throughout cannot be reached
     * by a formula change, and 315 paid by the formula cannot be reached by lowering a floor they
     * are not standing on. 253 + 315 = 568, and no district is in both.
     *
     * The two *moved* sets are not. 356 and 294 sum to 650 against 609 districts, so 41 move under
     * both — the districts the refresh lifts off the guarantee, which were also standing on the
     * floor the phase-out lowers. An earlier version of this test was named
     * `..._reach_disjoint_populations` and asserted `568` as though it proved a partition, when
     * 568 < 609 is the arithmetic disproof of one. The overlap is asserted here so the claim
     * cannot drift back.
     */
    let districts = panel();
    let refresh = simulate(
        &districts,
        &Policy {
            base_cost_scale: 1.0395,
            ..Policy::current_law()
        },
    );
    let phase_out = simulate(
        &districts,
        &Policy {
            guarantee: GuaranteeRule::PhasedOut { remaining: 0.5 },
            ..Policy::current_law()
        },
    );
    let both = price(
        &draft("fund-the-plan-and-retire-the-guarantee").unwrap(),
        &districts,
    );

    let total = districts.len();
    assert_eq!(total, 609);
    assert_eq!(refresh.unmoved(), 253);
    assert_eq!(phase_out.unmoved(), 315);

    // Disjoint where it is claimed: no district is unmoved by both, so the union of what they
    // reach is everybody.
    assert_eq!(refresh.unmoved() + phase_out.unmoved(), total - 41);
    assert_eq!(
        both.effect().unmoved(),
        0,
        "between them the two levers should leave nobody untouched"
    );

    // And overlapping where it is not. Computed rather than asserted as a literal, so the count
    // and the claim cannot come apart.
    let moved_by_both = (refresh.gainers() + phase_out.losers()).saturating_sub(total);
    assert_eq!(
        moved_by_both, 41,
        "the two levers move an overlapping set, not a partition"
    );
    assert_eq!(
        moved_by_both,
        refresh
            .policy
            .on_guarantee
            .abs_diff(phase_out.policy.on_guarantee)
            - 41
            + 41,
        "the overlap is the districts the refresh lifts off the floor the phase-out lowers"
    );
}

#[test]
fn a_lever_key_that_is_not_one_of_the_five_is_refused() {
    // The runnable surface is five fields on `Policy` and there is no sixth. A fixture row naming
    // a lever that does not exist has to fail loudly: silently treating it as unpriced would turn
    // a typo into a caveat, and the caveat would look deliberate.
    assert!(Lever::parse("special-ed-weight", "1.08").is_err());
    assert!(Lever::parse("base-cost", "not a number").is_err());
    assert!(Lever::parse("guarantee", "abolish").is_err());
    assert!(Lever::parse("guarantee", "phase-out:0.5").is_ok());
}

#[test]
fn applying_a_lever_leaves_the_other_four_alone() {
    // `Draft::policy` folds provisions over current law, so a lever that reset a field it was not
    // named for would make a draft's policy depend on the order its provisions are written in.
    let base = Lever::BaseCostScale(1.05).applied_to(Policy::current_law());
    assert!((base.base_cost_scale - 1.05).abs() < f64::EPSILON);
    assert_eq!(base.guarantee, Policy::current_law().guarantee);
    assert!(
        (base.minimum_state_share - Policy::current_law().minimum_state_share).abs() < f64::EPSILON
    );
}

#[test]
fn a_drafts_policy_does_not_depend_on_provision_order() {
    // Two provisions moving different levers commute. One moving the same lever twice would not,
    // and a draft doing that is an authoring error rather than a policy — so the fixture is
    // checked for it here rather than left to produce an order-dependent number.
    for (slug, draft) in drafts() {
        let mut seen = BTreeSet::new();
        for provision in draft.priced() {
            // `Lever::key` rather than a `match` written out again here. Its whole docstring is
            // that the five strings are written down once; a test that re-implemented the mapping
            // would be the second copy it warns about, and would go on passing after `key` broke.
            let key = provision
                .lever
                .expect("priced() only yields provisions with a lever")
                .key();
            assert!(
                seen.insert(key),
                "{slug} moves {key} in two provisions; the later one would silently win"
            );
        }
    }
}

#[test]
fn a_draft_nothing_prices_reports_no_cost_rather_than_a_cost_of_zero() {
    /*
     * The defect the first real pending bill found, and the reason `cost` is an `Option`.
     *
     * H.B. 643 of the 136th caps EdChoice expansion eligibility. Every provision it has falls in
     * the scholarship channel, which this workspace does not carry — so the combined policy is
     * *identical to current law* and the arithmetic gives exactly `0.00`. Returned as a bare
     * `Dollars` that reads as "this bill is free", which is the opposite of "this bill was not
     * priced".
     *
     * The shape is ordinary rather than exotic: it is what a bill about vouchers, transportation
     * or facilities looks like here, which is most of what touches school funding outside a
     * budget act.
     */
    let districts = panel();
    let priced = price(&draft("hb-643-136-introduced").unwrap(), &districts);

    assert!(priced.attribution().is_empty(), "no provision should price");
    assert_eq!(priced.unpriced().len(), 1);
    assert_eq!(
        priced.cost(),
        None,
        "not Some(0.0) — the claims are opposite"
    );
    assert_eq!(
        priced.residual(),
        None,
        "nothing for the parts to fail to sum to"
    );

    // And the underlying run really is current law, which is what makes the zero so plausible.
    assert!(priced.effect().cost().abs() < f64::EPSILON);
    assert_eq!(priced.effect().unmoved(), 609);
}

#[test]
fn the_pending_bill_names_the_version_and_digest_it_was_read_from() {
    /*
     * A pending bill's text moves under its own URL as it is amended, so a node written against
     * "H.B. 643" without a version names a moving target. `00_IN` is stable and the digest pins
     * the bytes; this asserts the node carries both rather than trusting that it does.
     */
    let text = node("hb-643-136-introduced");
    assert!(text.contains("00_IN"), "the version read is not named");
    let digests = std::fs::read_to_string(root().join("crates/connect/source-digests.txt"))
        .expect("the digest manifest");
    let pinned = digests
        .lines()
        .find(|line| line.ends_with("hb643-136-introduced"))
        .expect("the bill is not pinned in source-digests.txt");
    let sha = pinned.split_whitespace().next().expect("a digest");
    assert!(
        text.contains(sha),
        "the node cites a digest the manifest does not carry; manifest says {sha}"
    );
}

#[test]
fn every_lever_key_round_trips_through_parse_and_back() {
    /*
     * `Lever::key` is the inverse of `Lever::parse`, and its only production caller is the feed
     * emitter in `crates/bundle`, which writes the key the browser's query string then reads. So a
     * transposed arm — `PhaseInGeneral` returning `"phase-in-dpia"` — would survive every other
     * check here: the feed schema accepts any of the five, and the fixture exercises only two.
     *
     * The browser would then apply the categorical phase-in where the bill said base cost, and
     * publish a cost that is not the draft's. This is the check that fails first.
     */
    for (key, value) in [
        ("guarantee", "phase-out:0.5"),
        ("base-cost", "1.0395"),
        ("min-share", "0.05"),
        ("phase-in", "0.8333"),
        ("phase-in-dpia", "0.6"),
    ] {
        let lever = Lever::parse(key, value).unwrap_or_else(|e| panic!("{key}: {e}"));
        assert_eq!(lever.key(), key, "{key} does not survive the round trip");
    }
}

#[test]
fn each_lever_key_moves_the_field_it_names_and_no_other() {
    // The other half of the transposition guard: a key that round-trips could still be wired to
    // the wrong field of `Policy`. Five levers, five fields, checked one at a time.
    let base = Policy::current_law();
    let moved = |key: &str, value: &str| {
        Lever::parse(key, value)
            .unwrap_or_else(|e| panic!("{key}: {e}"))
            .applied_to(base)
    };

    assert_eq!(
        moved("guarantee", "removed").guarantee,
        GuaranteeRule::Removed
    );
    assert!((moved("base-cost", "1.05").base_cost_scale - 1.05).abs() < f64::EPSILON);
    assert!((moved("min-share", "0.05").minimum_state_share - 0.05).abs() < f64::EPSILON);
    assert!((moved("phase-in", "0.75").phase_in_general - 0.75).abs() < f64::EPSILON);
    assert!((moved("phase-in-dpia", "0.6").phase_in_dpia - 0.6).abs() < f64::EPSILON);

    // And each leaves the other four alone, which is what makes `Draft::policy`'s fold safe.
    let one = moved("phase-in", "0.75");
    assert!((one.phase_in_dpia - base.phase_in_dpia).abs() < f64::EPSILON);
    assert!((one.base_cost_scale - base.base_cost_scale).abs() < f64::EPSILON);
}

#[test]
fn a_nodes_unpriced_field_accounts_for_every_provision_the_fixture_could_not_price() {
    /*
     * The check the first version of `the_node_admits_what_the_fixture_could_not_price` only
     * appeared to make. Its `else` branch asserted that each unpriced provision's *authority*
     * appears somewhere in the node — which the `provisions:` property already satisfies, so
     * emptying `unpriced:` to "None." left every test passing.
     *
     * This reads the `unpriced:` block itself and requires each unpriced provision to be
     * accounted for inside it, so a node that priced two of five clauses and claimed nothing was
     * left out fails here.
     */
    for (slug, draft) in drafts() {
        let text = node(&slug);
        let block = unpriced_block(&text)
            .unwrap_or_else(|| panic!("{slug} has no `unpriced:` property; it is mandatory"));
        let unpriced = draft.unpriced();

        if unpriced.is_empty() {
            assert!(
                block.contains("None"),
                "{slug} prices every provision and its `unpriced:` block does not say so: {block:?}"
            );
            continue;
        }
        assert!(
            !block.trim_start().starts_with("None"),
            "{slug} cannot price {} provisions and its `unpriced:` block opens by saying none",
            unpriced.len()
        );
        for provision in unpriced {
            assert!(
                names(&block, &provision.authority),
                "{slug} provision {} is unpriced and the `unpriced:` block never names {}",
                provision.ordinal,
                provision.authority
            );
        }
    }
}

/// The body of a node's `unpriced:` block scalar, without parsing YAML.
///
/// Every property in this corpus is a block scalar indented four spaces under a two-space key, so
/// the block runs to the next line that is non-empty and less indented. Written here rather than
/// pulled in as a dependency because `crates/` has none, and a whole YAML parser to read one
/// property would be the wrong trade.
fn unpriced_block(text: &str) -> Option<String> {
    let mut lines = text.lines().skip_while(|l| !l.starts_with("  unpriced:"));
    lines.next()?;
    let body: Vec<&str> = lines
        .take_while(|l| l.trim().is_empty() || l.starts_with("    "))
        .collect();
    Some(body.join("\n"))
}

#[test]
fn the_lever_flags_and_the_draft_flag_name_the_same_five_things() {
    /*
     * The CLI refuses a lever flag beside `--draft`, and the refusal is only as good as its list.
     * A sixth lever added to `Policy` without a sixth entry there would be silently accepted
     * alongside a draft and attributed to the bill.
     *
     * There is no way to read the binary's `const LEVERS` from here, so this asserts the thing it
     * has to stay in step with: the five keys `Lever` knows. If that count changes, this fails and
     * whoever changed it goes and looks at `main.rs`.
     */
    let keys = [
        Lever::Guarantee(GuaranteeRule::AsEnacted).key(),
        Lever::BaseCostScale(1.0).key(),
        Lever::MinimumStateShare(0.1).key(),
        Lever::PhaseInGeneral(1.0).key(),
        Lever::PhaseInDpia(1.0).key(),
    ];
    let unique: BTreeSet<&str> = keys.into_iter().collect();
    assert_eq!(
        unique.len(),
        5,
        "the CLI's --draft guard lists five lever flags; keep them in step"
    );
    for key in keys {
        assert!(
            Lever::parse(key, "0.5").is_ok() || key == "guarantee",
            "{key} is not parseable, so the guard's flag list and Lever have diverged"
        );
    }
}
