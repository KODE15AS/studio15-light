# Lobbyen for Studio 15 LIGHT: Rust-backend (axum) + Svelte-frontend.
# Build- og runtime-stage bruker samme Debian-versjon (trixie) —
# glibc-lærdommen fra Skjermsamling.

FROM node:22.22-trixie-slim AS web
WORKDIR /app
COPY frontend/package.json .
RUN npm install --no-audit --no-fund
COPY frontend/ .
RUN npm run build

FROM rust:1.99-slim-trixie AS build
WORKDIR /app
COPY Cargo.toml .
COPY src src
RUN cargo build --release

FROM debian:trixie-slim
# git: seeding av prosjektenes bare-repoer på repos-volumet
RUN apt-get update \
  && apt-get install -y --no-install-recommends git ca-certificates \
  && rm -rf /var/lib/apt/lists/*
COPY --from=build /app/target/release/lobby /usr/local/bin/lobby
COPY --from=web /app/dist /opt/lobby/web
COPY prosjektmal /opt/prosjektmal

EXPOSE 8200
CMD ["lobby"]
