##################################################################################
# BEGIN base layers
##################################################################################
##################################################################################
FROM ghcr.io/containercraft/konductor:latest
SHELL ["/bin/bash", "-c", "-e"]

# Github Token is used for github api calls to get latest releases
# This is optional but helps during development due to rate limiting)
# Note that this is a docker argument and not an environment variable (ARG vs ENV)
# so it is not persisted in the image layers and is not available
# at runtime (only build time)
ARG GITHUB_TOKEN=${GITHUB_TOKEN}
# Set default bin directory for new packages
ARG BIN="/usr/local/bin"
# Set default binary install command
ARG INSTALL="install -m 755 -o root -g root"

# Common Dockerfile Container Build Functions
ARG apt_update="sudo apt-get update"
ARG apt_install="TERM=linux DEBIAN_FRONTEND=noninteractive sudo apt-get install -q --yes --purge --assume-yes --auto-remove --allow-downgrades -o Dpkg::Options::='--force-confdef' -o Dpkg::Options::='--force-confold'"
ARG apt_clean="sudo apt-get clean && sudo apt-get autoremove -y && sudo apt-get purge -y --auto-remove"
ARG curl="/usr/bin/curl --silent --show-error --tlsv1.2 --location"
ARG curl_github="/usr/bin/curl --silent --show-error --tlsv1.2 --request GET --header "Authorization: Bearer $GITHUB_TOKEN" --header "X-GitHub-Api-Version: 2022-11-28" --url --location"
ARG dir_clean="\
  sudo rm -rf \
  /var/lib/{apt,cache,log} \
  /usr/share/{doc,man,locale} \
  /var/cache/apt \
  /home/*/.cache \
  /root/.cache \
  /var/tmp/* \
  /tmp/* \
  "

##################################################################################
# END base layers
##################################################################################
##################################################################################
# BEGIN user defined layers
##################################################################################
# Add SI dependencies
ARG APT_PKGS="\
  zstd \
  docker-compose-plugin \
  "
RUN echo \
  && ${apt_update} \
  && bash -c "${apt_install} ${APT_PKGS}" \
  && bash -c "${apt_clean}" \
  && ${dir_clean} \
  && echo

##################################################################################
# Install Buck2
# - https://github.com/facebook/buck2
RUN echo \
  && export NAME=buck2 \
  && export ARCH=$(uname -m) \
  && export REPOSITORY=facebook/buck2 \
  && URL=$(curl -sLk "https://api.github.com/repos/${REPOSITORY}/releases" | jq -r ".[0].assets[].browser_download_url | select(endswith(\"buck2-$ARCH-unknown-linux-gnu.zst\"))") \
  && BINARY="${NAME}-${ARCH}-unknown-linux-gnu" \
  && PKG="${BINARY}.zst" \
  && TEST="${BIN}/${NAME} --version" \
  && echo "---------------------------------------------------------"\
  && echo "INFO[${NAME}] Installing:" \
  && echo "INFO[${NAME}]   Name:           ${NAME}" \
  && echo "INFO[${NAME}]   Binary:         ${BINARY}" \
  && echo "INFO[${NAME}]   Package:        ${PKG}" \
  && echo "INFO[${NAME}]   Architecture:   ${ARCH}" \
  && echo "INFO[${NAME}]   Source:         ${URL}" \
  && echo "---------------------------------------------------------"\
  && ${curl} "${URL}" --output "/tmp/${PKG}" \
  && zstd -d "/tmp/${PKG}" -o "/tmp/${BINARY}" \
  && sudo ${INSTALL} "/tmp/${BINARY}" "${BIN}/${NAME}" \
  && ${TEST} \
  && ${dir_clean}
