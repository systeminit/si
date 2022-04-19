ARG BUILDER_IMAGE=cache
ARG NODE_VERSION=16.13.1
ARG NGINX_VERSION=1.21.4

###########################################################################
# Cache Stage
###########################################################################
FROM node:$NODE_VERSION as cache
ARG NODE_VERSION

WORKDIR /usr/src
COPY . .

WORKDIR app/web
RUN npm ci

###########################################################################
# Builder Stage
###########################################################################
FROM $BUILDER_IMAGE as builder

WORKDIR /usr/src
COPY . .

WORKDIR app/web
RUN env && npm run build

###########################################################################
# Final Stage
###########################################################################
FROM nginx:$NGINX_VERSION-alpine as final

RUN rm -rf /etc/nginx/conf.d/default.conf /usr/share/nginx/html/*
COPY --from=builder /usr/src/app/web/nginx.conf /etc/nginx/conf.d/default.conf
COPY --from=builder /usr/src/app/web/nginx/* /etc/ssl/certs/
COPY --from=builder /usr/src/app/web/dist /usr/share/nginx/html
