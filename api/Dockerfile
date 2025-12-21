# Build stage
FROM oven/bun:1 AS builder

WORKDIR /app

COPY package.json bun.lock ./
RUN bun install --frozen-lockfile

COPY . .
RUN bun run build

# Production stage
FROM oven/bun:1-slim AS production

WORKDIR /app

COPY --from=builder /app/dist ./dist
COPY --from=builder /app/package.json ./
COPY --from=builder /app/bun.lock ./

RUN bun install --frozen-lockfile --production --ignore-scripts

ENV NODE_ENV=production
ENV HOST=0.0.0.0

EXPOSE 5005

CMD ["bun", "run", "dist/index.js"]
