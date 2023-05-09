ARG BUILDER_IMAGE=cache
ARG NODE_VERSION=18.16.0
ARG NGINX_VERSION=1.21.4
ARG PNPM_VERSION=8.5.0

###########################################################################
# Cache Stage
###########################################################################
FROM node:$NODE_VERSION as cache
ARG NODE_VERSION

RUN npm install -g pnpm@$PNPM_VERSION

WORKDIR /usr/src
COPY . .

WORKDIR /usr/src/app/web

# TODO: might want to call pnpm fetch here instead, and might want to filter
# will investigate when we start breaking things out of web into monorepo
RUN set -eux; \
    pnpm install;

###########################################################################
# Builder Stage
###########################################################################
# hadolint ignore=DL3006
FROM $BUILDER_IMAGE as builder

WORKDIR /usr/src
COPY . .

WORKDIR /usr/src/app/web
RUN set -eux; \
    env; \
    pnpm run build;

###########################################################################
# Final Stage
###########################################################################
FROM nginx:$NGINX_VERSION-alpine as final

RUN set -eux; \
    rm -rf /etc/nginx/conf.d/default.conf /usr/share/nginx/html/*;

COPY --from=builder /usr/src/app/web/nginx.conf /etc/nginx/conf.d/default.conf
COPY --from=builder /usr/src/app/web/nginx/* /etc/ssl/certs/
COPY --from=builder /usr/src/app/web/dist /usr/share/nginx/html
