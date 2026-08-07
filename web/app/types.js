/**
 * The shape of `data/bundle.json`, as `crates/bundle` writes it.
 *
 * The field names are the contract. `CONTRACT_VERSION` in that crate is bumped whenever any of
 * them changes meaning, and {@link REQUIRED_CONTRACT} below is what this page will render.
 */
/** The bundle contract this page understands. */
export const REQUIRED_CONTRACT = "2.0.0";
