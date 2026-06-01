# ── Stage 1 : build ──────────────────────────────────────────────────────────
FROM rust:1-slim-bookworm AS builder

WORKDIR /app

# Dépendances système pour la compilation (OpenSSL headers non nécessaires car rustls)
RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config \
    && rm -rf /var/lib/apt/lists/*

# Pré-téléchargement des dépendances (couche cachée si Cargo.toml inchangé)
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs \
    && cargo build --release \
    && rm -rf src

# Compilation du vrai code source
COPY src ./src
# Touch main.rs pour forcer la recompilation du binaire final
RUN touch src/main.rs && cargo build --release

# ── Stage 2 : runtime ────────────────────────────────────────────────────────
FROM debian:bookworm-slim

# CA certificates pour les requêtes HTTPS (reqwest + rustls)
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/minebox-bot ./minebox-bot

# Utilisateur non-root pour la sécurité
RUN useradd -r -s /bin/false botuser && chown botuser:botuser /app/minebox-bot
USER botuser

CMD ["./minebox-bot"]
