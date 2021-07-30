ARG BUILDER_IMAGE=cache
ARG NODE_VERSION=16.5.0
ARG NGINX_VERSION=1.21.1

###########################################################################
# Cache Stage
###########################################################################
FROM node:$NODE_VERSION as cache
ARG NODE_VERSION

WORKDIR /usr/src
COPY . .

RUN (cd components/si-registry && npm ci && npm run build)
RUN (cd components/si-entity && npm ci && npm run build)
RUN (cd components/si-web-app && npm ci)

###########################################################################
# Builder Stage
###########################################################################
FROM $BUILDER_IMAGE as builder
ARG VUE_APP_SDF_BASE_HTTP_URL
ARG VUE_APP_SDF_BASE_WS_URL

WORKDIR /usr/src
COPY . .

WORKDIR components/si-web-app
RUN env && npm run build

###########################################################################
# Final Stage
###########################################################################
FROM nginx:$NGINX_VERSION-alpine as final

COPY --from=builder /usr/src/components/si-web-app/dist /usr/share/nginx/html
