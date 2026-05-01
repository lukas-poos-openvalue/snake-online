FROM cgr.dev/chainguard/node:latest-dev AS builder

ARG PROFILE="prod"

WORKDIR /app

COPY package.json package-lock.json* ./
RUN --mount=type=cache,target=/root/.npm npm ci
COPY . .
RUN npm run build-${PROFILE}


FROM lipanski/docker-static-website:2.6.0 AS runner

COPY --from=builder app/dist/frontend/browser .
