import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import tailwindcss from "@tailwindcss/vite";
import path from "node:path";

// Vite config:
// - React 19 via @vitejs/plugin-react
// - Tailwind v4 via the first-party Vite plugin (CSS-first, no JS config)
// - "@/..." import alias resolves to src/
// - /api proxy: SPA on :5173 forwards /api/* to the Rust server on :3000
//   so we avoid CORS hassles in dev. In prod, set VITE_API_BASE to your real API origin.

export default defineConfig({
    plugins: [react(), tailwindcss()],
    resolve: {
        alias: {
            "@": path.resolve(__dirname, "./src"),
        },
    },
    server: {
        port: 5173,
        proxy: {
            "/api": {
                target: "http://127.0.0.1:3000",
                changeOrigin: true,
                rewrite: (p) => p.replace(/^\/api/, ""),
            },
        },
    },
});
