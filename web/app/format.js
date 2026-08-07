/** Number and text formatting. Small enough to test exhaustively, and it has been wrong. */
/** A dollar amount, or an em dash where there is no value. */
export function money(v, decimals = 0) {
    if (v == null || !Number.isFinite(v))
        return "—";
    return ("$" +
        v.toLocaleString("en-US", {
            minimumFractionDigits: decimals,
            maximumFractionDigits: decimals,
        }));
}
/** A signed dollar amount, so a zero change reads as zero rather than as a gain. */
export function signedMoney(v, decimals = 0) {
    if (v == null || !Number.isFinite(v))
        return "—";
    if (Math.abs(v) < 0.5 * Math.pow(10, -decimals))
        return money(0, decimals);
    return (v > 0 ? "+" : "−") + money(Math.abs(v), decimals);
}
/**
 * A large dollar amount at a readable scale.
 *
 * Switches to billions past a thousand million, because "$7281.2M" is a number a reader has to
 * count the digits of and "$7.28B" is one they can hold.
 */
export function millions(v) {
    if (v == null || !Number.isFinite(v))
        return "—";
    const sign = v < 0 ? "−" : v > 0 ? "+" : "";
    const magnitude = Math.abs(v);
    return magnitude >= 1_000_000_000
        ? `${sign}$${(magnitude / 1_000_000_000).toFixed(2)}B`
        : `${sign}$${(magnitude / 1_000_000).toFixed(1)}M`;
}
/** A fraction as a percentage. */
export function pct(v, decimals = 1) {
    if (v == null || !Number.isFinite(v))
        return "—";
    return (v * 100).toFixed(decimals) + "%";
}
/** A count with thousands separators. */
export function count(v) {
    return v.toLocaleString("en-US", { maximumFractionDigits: 0 });
}
/**
 * 1st, 2nd, 3rd, 4th … 11th, 12th, 13th … 21st.
 *
 * The teens are the whole difficulty, and getting them wrong put "2th percentile" on this page
 * once. It was caught by looking at the rendered output, not by any test — hence the tests.
 */
export function ordinal(n) {
    const teens = n % 100;
    if (teens >= 11 && teens <= 13)
        return `${n}th`;
    const suffix = { 1: "st", 2: "nd", 3: "rd" }[n % 10] ?? "th";
    return `${n}${suffix}`;
}
/** Share of `values` strictly below `v`. */
export function percentileOf(sorted, v) {
    let below = 0;
    for (const x of sorted) {
        if (x < v)
            below++;
        else
            break;
    }
    return sorted.length === 0 ? 0 : below / sorted.length;
}
/** Escape text for interpolation into HTML. District names are data, not markup. */
export function escapeHtml(s) {
    return s.replace(/[&<>"']/g, (c) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#39;" })[c]);
}
