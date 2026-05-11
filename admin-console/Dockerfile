# ── 构建阶段 ──────────────────────────────────────────
FROM node:22-alpine AS builder

WORKDIR /app

COPY package.json package-lock.json ./
RUN npm ci

ARG BACKEND_URL=http://backend:8744
ENV BACKEND_URL=${BACKEND_URL}

COPY tsconfig.json next.config.ts ./
COPY app ./app

RUN npm run build

# ── 运行时阶段 ──────────────────────────────────────────
FROM node:22-alpine

RUN adduser --no-create-home --shell /bin/sh appuser

WORKDIR /app

COPY --from=builder /app/.next/standalone ./
COPY --from=builder /app/.next/static ./.next/static
COPY --from=builder /app/public ./public

RUN chown -R appuser:appuser /app
USER appuser

ENV HOSTNAME=0.0.0.0
ENV PORT=3000

EXPOSE 3000

CMD ["node", "server.js"]
