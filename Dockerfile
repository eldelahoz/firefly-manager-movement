# Single image: axum serves /api and the built frontend on :3000 (Firefly III stays external).

FROM node:22-slim AS frontend
WORKDIR /app/frontend
RUN corepack enable
COPY frontend/package.json frontend/pnpm-lock.yaml ./
RUN corepack prepare pnpm@11.1.2 --activate && pnpm install --frozen-lockfile
COPY frontend/ ./
RUN pnpm build

FROM rust:1.97-slim-bookworm AS backend
WORKDIR /app/backend
COPY backend/ ./
RUN cargo build --release --locked --bin ffm-backend

FROM debian:bookworm-slim
COPY --from=backend /app/backend/target/release/ffm-backend /usr/local/bin/ffm-backend
COPY --from=frontend /app/frontend/dist /app/dist
ENV DATABASE_URL=/data/ffm.db \
    FRONTEND_DIST=/app/dist \
    RUST_LOG=info
VOLUME /data
EXPOSE 3000
CMD ["ffm-backend"]
