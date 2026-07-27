/* =============================================================================
 * Shared Recharts theme.
 *
 * Recharts ships `itemStyle: { color: '#000' }` as a default and only falls back
 * to a series colour when one exists (DefaultTooltipContent.js). A <Bar> painted
 * per-<Cell> has no series colour, so every tooltip value rendered as black text
 * on our dark panel — 1.15:1, effectively invisible. Overriding `contentStyle`
 * alone does not fix it; `itemStyle` and `labelStyle` must be set too.
 *
 * Defining it once here also stops the four panels from drifting apart.
 * ========================================================================== */

/** Palette mirror of the @theme tokens in index.css. SVG attributes need real
 *  values, not utility classes, so these two must be kept in step. */
export const chart = {
    grid: "#1d2735",
    axis: "#8b9bb0",
    fg: "#d3dde8",
    fgDim: "#8b9bb0",
    accent: "#3dff92",
    critical: "#ff6b6b",
    high: "#ff9d52",
    tooltipBg: "#10151d",
    tooltipLine: "#2e3b4d",
    equator: "#33415a",
} as const;

/** 11px, not 9px: axis labels are read through LinkedIn's video transcode. */
export const axisTick = { fill: chart.axis, fontSize: 11 } as const;

export const axisLine = { stroke: chart.grid } as const;

/** Spread onto every <Tooltip>. */
export const tooltip = {
    contentStyle: {
        background: chart.tooltipBg,
        border: `1px solid ${chart.tooltipLine}`,
        borderRadius: 0,
        padding: "8px 10px",
        fontSize: 12,
        fontFamily: "var(--font-mono)",
        boxShadow: "0 8px 24px rgba(0,0,0,0.55)",
    },
    itemStyle: {
        color: chart.fg,
        fontSize: 12,
        paddingTop: 2,
        paddingBottom: 2,
    },
    labelStyle: {
        color: chart.fgDim,
        fontSize: 10,
        letterSpacing: "1.5px",
        textTransform: "uppercase" as const,
        marginBottom: 4,
    },
    cursor: { fill: "rgba(255,255,255,0.05)" },
} as const;

/** Line/area charts want a crosshair rather than a filled band. */
export const tooltipLineCursor = {
    ...tooltip,
    cursor: { stroke: chart.fgDim, strokeWidth: 1, strokeDasharray: "3 3" },
} as const;
