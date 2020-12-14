FROM systeminit/si-builder:latest as build
LABEL maintainer="dev@systeminit.com"
COPY . /src
WORKDIR /src
ENV NPM_CONFIG_ALLOW_ROOT=true
ENV NPM_CONFIG_UNSAFE_PERM=true
ENV VUE_APP_SDF=https://internal-api.systeminit.com
ENV VUE_APP_SDF_WS=wss://internal-api.systeminit.com/updates
RUN make build//components/si-web-app

FROM nginx:mainline-alpine
LABEL maintainer="dev@systeminit.com"
COPY --from=build /src/components/si-web-app/dist /app
COPY --from=build /src/components/si-web-app/htpasswd /etc/nginx/htpasswd
COPY --from=build /src/components/si-web-app/nginx.conf /etc/nginx/nginx.conf
