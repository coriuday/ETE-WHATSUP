# ─────────────────────────────────────────────
# Next.js 15 Frontend Dockerfile
# ─────────────────────────────────────────────

FROM oven/bun:1-alpine AS builder

WORKDIR /app

COPY package.json bun.lockb* ./
RUN bun install

COPY . .

# Set env placeholder for build time
ENV NEXT_PUBLIC_API_URL=http://localhost:80/api/v1
ENV NEXT_PUBLIC_WS_URL=ws://localhost:80/api/v1/ws

RUN bun run build

# Runner Stage
FROM oven/bun:1-alpine AS runner

WORKDIR /app

ENV NODE_ENV=production

# Copy built package and modules
COPY --from=builder /app/package.json ./package.json
COPY --from=builder /app/node_modules ./node_modules
COPY --from=builder /app/.next ./.next
COPY --from=builder /app/public ./public
COPY --from=builder /app/next.config.ts ./next.config.ts

EXPOSE 3000

CMD ["bun", "run", "start"]
