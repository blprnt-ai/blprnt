FROM node:22-bookworm AS frontend-build
WORKDIR /app/frontend

COPY frontend/package.json frontend/pnpm-lock.yaml ./
RUN corepack enable && pnpm install --frozen-lockfile

COPY frontend/ ./
COPY scripts/ /app/scripts/
RUN pnpm build

FROM rust:1.90-bookworm AS backend-build
WORKDIR /app/backend

COPY backend/Cargo.toml backend/Cargo.lock ./
COPY backend/.cargo ./.cargo
COPY backend/crates ./crates
RUN apt-get update
RUN apt-get install -y build-essential clang libclang-dev lld pkg-config
RUN cargo build --profile docker-release --locked -p blprnt

FROM debian:bookworm-slim AS runtime
RUN apt-get update \
  && apt-get install -y --no-install-recommends ca-certificates libstdc++6 \
  && rm -rf /var/lib/apt/lists/*

WORKDIR /opt/blprnt

COPY --from=backend-build /app/backend/target/docker-release/blprnt /usr/local/bin/blprnt
COPY --from=frontend-build /app/frontend/dist ./dist

ENV BLPRNT_BASE_DIR=/opt/blprnt/dist
ENV BLPRNT_DEPLOYED=true
ENV HOME=/var/lib/blprnt-home
ENV BLPRNT_API_PORT=9171
ENV BLPRNT_OPEN_BROWSER=false

RUN mkdir -p /var/lib/blprnt-home

EXPOSE 9171

CMD ["blprnt"]
