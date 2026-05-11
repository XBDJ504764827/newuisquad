# ── 构建阶段 ──────────────────────────────────────────
FROM rust:1.89-slim-bookworm AS builder

RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config libssl-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# 预取并编译依赖（利用 Docker 层缓存）
COPY Cargo.toml Cargo.lock ./
COPY .cargo .cargo
# 创建临时 lib.rs 占位所有模块声明，确保依赖完整编译
RUN echo 'pub mod api; pub mod config; pub mod db; pub mod models; pub mod protocol; pub mod redis; pub mod repositories; pub mod rcon_client; pub mod services;' > src/lib.rs \
    && echo 'fn main() {}' > src/main.rs \
    && mkdir -p src/api src/models src/rcon_client src/repositories src/services \
    && for d in api models rcon_client repositories services; do echo '' > src/$d/mod.rs; done
RUN cargo build --release --bin admin-console-backend
RUN rm -rf src

# 编译真实源码
COPY src ./src
COPY migrations ./migrations
RUN cargo build --release --bin admin-console-backend

# ── 运行时阶段 ──────────────────────────────────────────
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

RUN useradd --no-create-home --shell /bin/sh appuser

WORKDIR /app

COPY --from=builder /app/target/release/admin-console-backend .
COPY --from=builder /app/migrations ./migrations

RUN chown -R appuser:appuser /app
USER appuser

EXPOSE 8744

CMD ["./admin-console-backend"]
