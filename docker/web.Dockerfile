# syntax=docker/dockerfile:1
# Build the React/Vite console, then serve it from Caddy (which also reverse-
# proxies /api → the API, giving a single origin so SSE/WS/cookies just work).
# Build context is the repo ROOT — see the root .dockerignore.

FROM node:22-bookworm-slim AS build
WORKDIR /web

# Install dependencies from the lockfile first so this layer is cached.
COPY aegis-console/package.json aegis-console/package-lock.json ./
RUN npm ci

# Copy ONLY the sources. Never `COPY aegis-console/ ./` — that would drop the
# host's node_modules on top of the freshly installed one, mixing platform-
# specific binaries (rollup/lightningcss) and breaking the build.
COPY aegis-console/index.html      ./
COPY aegis-console/vite.config.ts  ./
COPY aegis-console/tsconfig.json aegis-console/tsconfig.app.json aegis-console/tsconfig.node.json ./
COPY aegis-console/public ./public
COPY aegis-console/src    ./src

RUN npm run build

FROM caddy:2-alpine
COPY docker/Caddyfile /etc/caddy/Caddyfile
COPY --from=build /web/dist /srv
