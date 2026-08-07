/**
 * The funding formula, re-derived in the browser.
 *
 * This is a second implementation of `crates/project/src/policy.rs`. Two implementations of the
 * same formula is normally a bad trade — they drift, and the one nobody runs is the one that is
 * wrong. It is acceptable here for one reason: **the feed carries Rust-computed checkpoints and
 * this page refuses to render a scenario until it reproduces every one of them**, against the
 * real 609-district panel, on every load. See `verify.ts`.
 *
 * So the Rust stays authoritative and the TypeScript has to prove it agrees. If someone edits
 * one and not the other, the page says so instead of showing a plausible wrong number.
 *
 * Keep this file a line-for-line mirror of `apply()` in the Rust. Any cleverness here is a
 * liability.
 */
/** Current law: the identity, which reproduces the department's own FY2027 model. */
export function currentLaw(modelMinimumStateShare) {
    return {
        guarantee: { kind: "as-enacted" },
        baseCostScale: 1,
        minimumStateShare: modelMinimumStateShare,
        phaseInBaseCost: 1,
        phaseInCategorical: 1,
    };
}
/** Formula aid under current law: base cost share plus every categorical. */
function currentFormulaAid(d) {
    return d.base_cost_state_share + d.categorical_funding;
}
/**
 * Realized aid under current law.
 *
 * Summed from the components rather than taken from `realized_aid_per_pupil × adm`, which would
 * lose cents to the round trip and make the identity only approximately the identity.
 */
export function currentRealizedAid(d) {
    return currentFormulaAid(d) + d.guarantee;
}
/**
 * Apply a policy to one district at a given current-year enrolled ADM.
 *
 * `modelMinimumStateShare` is the minimum the *published model* was computed under — 10% for
 * FY2027 — and is distinct from `policy.minimumStateShare`, which is the one being proposed.
 * The two are only equal under current law.
 */
export function apply(d, p, currentYearAdm, modelMinimumStateShare) {
    const baseCostPerPupil = d.base_cost_per_pupil * p.baseCostScale;
    const floorPerPupil = baseCostPerPupil * p.minimumStateShare;
    const modelledSharePerPupil = d.current_year_adm > 0 ? d.base_cost_state_share / d.current_year_adm : 0;
    const increasePerPupil = baseCostPerPupil - d.base_cost_per_pupil;
    // A district at the minimum state share has its local capacity censored by the floor: all
    // that is known is that it exceeds `1 − minimum` of base cost, not by how much. It is held at
    // the floor, which understates the gain for any whose capacity is only just above.
    const censored = d.at_minimum_state_share;
    const residualPerPupil = modelledSharePerPupil + increasePerPupil;
    const atMinimum = censored || residualPerPupil < floorPerPupil;
    const admRatio = d.current_year_adm > 0 ? currentYearAdm / d.current_year_adm : 1;
    let baseCostAid;
    if (censored) {
        baseCostAid =
            d.base_cost_state_share *
                admRatio *
                p.baseCostScale *
                (p.minimumStateShare / modelMinimumStateShare);
    }
    else if (residualPerPupil < floorPerPupil) {
        baseCostAid = floorPerPupil * currentYearAdm;
    }
    else {
        // Dollar-for-dollar per pupil, not proportional: local capacity does not move when base
        // cost does, so the state's residual absorbs the whole per-pupil increase.
        baseCostAid =
            d.base_cost_state_share * admRatio + increasePerPupil * currentYearAdm;
    }
    baseCostAid *= p.phaseInBaseCost;
    const formulaAid = baseCostAid + d.categorical_funding * p.phaseInCategorical * admRatio;
    const baseline = d.on_guarantee ? currentRealizedAid(d) : 0;
    let heldAt;
    switch (p.guarantee.kind) {
        case "as-enacted":
            heldAt = baseline;
            break;
        case "rebase":
            heldAt = baseline * p.guarantee.factor;
            break;
        case "phase-out":
            heldAt = formulaAid + p.guarantee.remaining * Math.max(0, baseline - formulaAid);
            break;
        case "removed":
            heldAt = 0;
            break;
    }
    const realizedAid = Math.max(formulaAid, heldAt);
    const baselineRealizedAid = currentRealizedAid(d);
    const delta = realizedAid - baselineRealizedAid;
    return {
        irn: d.irn,
        name: d.name,
        adm: currentYearAdm,
        formulaAid,
        realizedAid,
        guarantee: Math.max(0, realizedAid - formulaAid),
        baselineRealizedAid,
        onGuarantee: realizedAid > formulaAid + 0.005,
        atMinimumStateShare: atMinimum,
        delta,
        deltaPerPupil: currentYearAdm > 0 ? delta / currentYearAdm : 0,
    };
}
/** Apply a policy across the panel at modelled enrollment. */
export function applyAll(districts, p, modelMinimumStateShare) {
    return districts.map((d) => apply(d, p, d.current_year_adm, modelMinimumStateShare));
}
/** Aggregate a set of outcomes. */
export function totals(outcomes) {
    let realizedAid = 0;
    let formulaAid = 0;
    let guarantee = 0;
    let baseline = 0;
    let onGuarantee = 0;
    let atMinimumStateShare = 0;
    let gainers = 0;
    let losers = 0;
    for (const o of outcomes) {
        realizedAid += o.realizedAid;
        formulaAid += o.formulaAid;
        guarantee += o.guarantee;
        baseline += o.baselineRealizedAid;
        if (o.onGuarantee)
            onGuarantee++;
        if (o.atMinimumStateShare)
            atMinimumStateShare++;
        if (o.delta > 0.005)
            gainers++;
        else if (o.delta < -0.005)
            losers++;
    }
    return {
        districts: outcomes.length,
        onGuarantee,
        atMinimumStateShare,
        realizedAid,
        formulaAid,
        guarantee,
        cost: realizedAid - baseline,
        gainers,
        losers,
        unmoved: outcomes.length - gainers - losers,
    };
}
