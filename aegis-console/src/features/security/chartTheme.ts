/* =============================================================================
 * Palette mirror of the @theme tokens in index.css.
 *
 * SVG attributes need literal values, not utility classes, so these must be
 * kept in step with index.css by hand. Only the geo map still needs them —
 * every other panel is CSS bars now, drawn with real Tailwind utilities.
 * ========================================================================== */

export const chart = {
    grid: "#d7d3d3",        /* neutral-300 — graticule */
    equator: "#9b9797",     /* neutral-500 — the one anchored line */
    fg: "#201e1d",
    fgDim: "#605d5d",
    marker: "#444141",      /* neutral-800 — an idle origin */
    accent: "#ec3013",      /* a hot origin, and every impossible-travel arc */
    paper: "#f8f4f4",
} as const;
