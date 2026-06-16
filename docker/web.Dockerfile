# syntax=docker/dockerfile:1
# Build the React/Vite console, then serve it from Caddy (which also reverse-
# proxies /api → the API, giving a single origin so SSE/WS/cookies just work).
# Build context is the repo root.

FROM node:22-bookworm-slim AS build
WORKDIR /web
COPY aegis-console/package.json aegis-console/package-lock.json* ./
RUN npm ci || npm install
COPY aegis-console/ ./
RUN npm run build

FROM caddy:2-alpine
COPY docker/Caddyfile /etc/caddy/Caddyfile
COPY --from=build /web/dist /srv
