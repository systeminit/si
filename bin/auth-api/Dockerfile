ARG NODE_VERSION=18.16.0
ARG PNPM_VERSION=8.1.1
ARG PACKAGE_PATH=@si/auth-api

FROM node:$NODE_VERSION-slim AS base
# Check https://github.com/nodejs/docker-node/tree/b4117f9333da4138b03a546ec926ef50a31506c3#nodealpine to understand why libc6-compat might be needed.
# RUN apk add --no-cache libc6-compat
# RUN apk update

# install pnpm
# RUN wget -qO- https://get.pnpm.io/install.sh | ENV="$HOME/.shrc" SHELL="$(which sh)" sh -

WORKDIR /app

RUN npm i -g turbo
COPY . .
RUN turbo prune --scope=@si/auth-api --docker

FROM node:$NODE_VERSION-slim AS installer
WORKDIR /app
COPY .gitignore .gitignore
COPY --from=base /app/out/json/ ./
COPY --from=base /app/out/pnpm-lock.yaml ./app/out/pnpm-workspace.yaml ./
RUN npm i -g pnpm@$PNPM_VERSION

RUN --mount=type=cache,id=pnpm-store,target=/root/.pnpm-store\
  # ↑ By caching the content-addressable store we stop downloading the same packages again and again
  PRISMA_SKIP_POSTINSTALL_GENERATE=true\
  pnpm install --filter . --filter @si/auth-api... --frozen-lockfile\
  --unsafe-perm\
  # ↑ Docker runs pnpm as root and then pnpm won't run package scripts unless we pass this arg
  | grep -v "cross-device link not permitted\|Falling back to copying packages from store"

# ↑ This inverted grep match is because using Docker's 'cache' mount type
# causes Docker to place the pnpm content-addressable store on a different virtual drive,
# which prohibits pnpm from symlinking its content to its virtual store (in node_modules/.pnpm),
# and that causes pnpm to fall back on copying the files. And that's fine,
# except pnpm emits a lot of warnings doing that, so here we filter out those warnings.

COPY --from=base /app/out/full/ .

# # Set working directory
# WORKDIR /app
# COPY package.json pnpm-lock.yaml pnpm-workspace.yaml ./

# # pnpm filtering here doesnt work :(
# # TODO: use local cache maybe?
# # RUN pnpm fetch
# COPY lib/tsconfig lib/tsconfig
# COPY lib/eslint-config lib/eslint-config
# COPY lib/ts-lib lib/ts-lib
# COPY bin/auth-api ./bin/auth-api

# # Calling pnpm fetch first would be better if we had a shared base image and were building multiple images
# # RUN --mount=type=cache,id=pnpm-store,target=/root/.pnpm-store\
# #   pnpm fetch
# # RUN PRISMA_SKIP_POSTINSTALL_GENERATE=1 pnpm install --recursive --offline --frozen-lockfile --unsafe-perm --filter . --filter @si/auth-api...

# # but we only care about this project, so we'll call install instead
# RUN --mount=type=cache,id=pnpm-store,target=/root/.pnpm-store\
#   # ↑ By caching the content-addressable store we stop downloading the same packages again and again
#   PRISMA_SKIP_POSTINSTALL_GENERATE=true\
#   pnpm install --filter . --filter @si/auth-api... --frozen-lockfile\
#   --unsafe-perm\
#   # ↑ Docker runs pnpm as root and then pnpm won't run package scripts unless we pass this arg
#   | grep -v "cross-device link not permitted\|Falling back to copying packages from store"

#   # ↑ This inverted grep match is because using Docker's 'cache' mount type
#   # causes Docker to place the pnpm content-addressable store on a different virtual drive,
#   # which prohibits pnpm from symlinking its content to its virtual store (in node_modules/.pnpm),
#   # and that causes pnpm to fall back on copying the files. And that's fine,
#   # except pnpm emits a lot of warnings doing that, so here we filter out those warnings.

# # RUN pnpm install --frozen-lockfile --filter . --filter @si/auth-api...

# # # Don't run production as root
# # RUN addgroup --system --gid 1001 koa
# # RUN adduser --system --uid 1001 koa
# # USER koa
# # COPY --from=installer /app .

FROM installer as builder
WORKDIR /app/bin/auth-api
RUN pnpx prisma generate
RUN pnpm run build

RUN PRISMA_SKIP_POSTINSTALL_GENERATE=1 pnpm --prod --filter @si/auth-api deploy pruned
# RUN cp -r src/prisma-client dist
RUN rm -rf pruned/src

FROM node:$NODE_VERSION-alpine as runner
RUN PRISMA_SKIP_POSTINSTALL_GENERATE=1 npm i -g pnpm@$PNPM_VERSION

WORKDIR /app

COPY ./config ./config

# # ENV NODE_ENV=production
COPY --from=builder /app/bin/auth-api/pruned .
COPY --from=builder /app/bin/auth-api/dist/src ./dist

# have to deal with our @si/ts-lib code which exports raw typescript but has been built by this build process
# this is a bit messy, but have tried lots of other solutions to get it running and failed
# move built js files from dist folder into node_modules
COPY --from=builder /app/bin/auth-api/dist/node_modules/@si/ts-lib ./node_modules/@si/ts-lib
# update package.json to point to the js files instead of ts
RUN sed -i.bak "s/\.ts/\.js/g" ./node_modules/@si/ts-lib/package.json

# have to re-run, as it seems the prune call wipes the generated prisma client...
# TODO figure out the best way to avoid this
RUN pnpx prisma generate

# also prisma adds a huge amount of disk space...
# some discussion and possible things to try here - https://github.com/prisma/prisma/issues/11577

# # ENTRYPOINT ["node", "index.js"]
CMD pnpm run boot
