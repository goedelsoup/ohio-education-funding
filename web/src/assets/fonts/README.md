# The one font binary

`ohio-math-fallback.woff2` — 27,220 bytes — is the only typeface this repository ships. Everything
else on the site is set in whatever the reader's platform calls a UI sans, a mono or a serif, and
`../../styles/tokens/typography.css` explains at length why that is a standing decision rather than
an unfinished one.

## What it is

A 174-codepoint subset of **STIX Two Math 2.12 b168a**, cut from `@fontsource/stix-two-math` — a
devDependency, so the source is pinned by `pnpm-lock.yaml` rather than being a file that happened to
exist on somebody's machine. It keeps the `MATH` table and the vertical glyph assemblies inside it,
which is the only reason a delimiter can be drawn taller than a line.

It is renamed, for two reasons that point the same way. Functionally, `--font-math` already names
"STIX Two Math" for the readers who have it installed, and an `@font-face` under that name would
shadow their local copy — so a macOS reader would fetch this subset and lose the other 4,800 glyphs.
Legally, the OFL here declares no Reserved Font Name, but STIX Two is a trademark of the IEEE, and a
font with four fifths of its glyphs removed should not go out under somebody else's mark.

## Licence

SIL Open Font License 1.1 — `OFL.txt`, and also inside the font's own `name` table (ids 0, 13 and
14), where it survives the file being copied out of this repository. Adding those records back is a
step in the build script: harfbuzz drops every name id above 6.

## Rebuilding it

```
node ../../../scripts/subset-math-font.ts            # report, and diff against the committed file
node ../../../scripts/subset-math-font.ts --write    # rebuild
```

The script is the argument, not just the recipe: it records what the repertoire is, why the
Mathematical Italic block is not optional, and what `ssty` would have cost. The build is
byte-reproducible against the lockfile, so the no-argument form tells you whether the committed file
is still the one the recipe describes.

Widening the repertoire means editing `REPERTOIRE` in `../../lib/math-font.ts` and rebuilding. The
end-to-end suite walks every formula in `dist/` and fails if the corpus has written a character the
font cannot draw, so that need will announce itself rather than showing up as one glyph in the
wrong face.
