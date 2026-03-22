FROM cgr.dev/chainguard/node:latest-dev AS builder

ARG PROFILE=""

WORKDIR /app

COPY package.json package-lock.json* ./
RUN --mount=type=cache,target=/root/.npm npm ci
COPY . .
RUN [ -z "$PROFILE" ] || npm run build-${PROFILE} && npm run build


FROM docker.io/lipanski/docker-static-website:2.6.0 AS runner

COPY --from=builder app/dist/frontend/browser .

CMD ["/busybox-httpd", "-f", "-v", "-p", "3001"]
