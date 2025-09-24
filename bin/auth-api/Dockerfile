ARG NODE_VERSION=20.19.0
ARG PACKAGE_PATH=@si/auth-api

FROM node:$NODE_VERSION-slim AS base
WORKDIR /app
RUN npm i -g turbo
COPY . .
RUN turbo prune --scope=@si/auth-api --docker

FROM node:$NODE_VERSION-slim AS installer
WORKDIR /app
COPY --from=base /app/out/json/ ./
COPY --from=base /app/out/pnpm-lock.yaml ./app/out/pnpm-workspace.yaml ./
RUN npm i -g pnpm@^8
COPY --from=base /app/out/full/ .

FROM installer AS builder
WORKDIR /app/bin/auth-api
RUN apt-get update -y && apt-get install -y openssl \
    && pnpm --package=prisma@5.20.0 dlx prisma generate
RUN pnpm run build \
    && echo "strict-peer-dependencies=false" > .npmrc \
    && PRISMA_SKIP_POSTINSTALL_GENERATE=1 pnpm --prod --filter @si/auth-api deploy pruned \
    && rm -rf pruned/src

FROM node:$NODE_VERSION-alpine AS runner
RUN  npm i -g pnpm@^8 tsx
WORKDIR /app
COPY ./config ./config
COPY --from=builder /app/bin/auth-api/pruned .
COPY --from=installer /app/bin/auth-api/src ./src
RUN pnpm --package=prisma@5.20.0 dlx prisma generate
CMD pnpm run boot