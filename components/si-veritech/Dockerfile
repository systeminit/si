ARG NODE_VERSION=16.6.1

###########################################################################
# AWS CLI Stage
###########################################################################
FROM node:$NODE_VERSION as awscli
ARG AWSCLI_VERSION=2.0.30

WORKDIR /tmp
RUN curl -L https://awscli.amazonaws.com/awscli-exe-linux-x86_64-$AWSCLI_VERSION.zip \
    -o awscliv2.zip \
  && unzip awscliv2.zip \
  && ./aws/install --bin-dir /usr/local/aws-cli-bin

###########################################################################
# kubectl Stage
###########################################################################
FROM node:$NODE_VERSION as kubectl
ARG KUBECTL_VERSION=1.21.3

WORKDIR /tmp
RUN curl -LO https://dl.k8s.io/release/v$KUBECTL_VERSION/bin/linux/amd64/kubectl \
  && curl -LO https://dl.k8s.io/release/v$KUBECTL_VERSION/bin/linux/amd64/kubectl.sha256 \
  && echo "$(cat kubectl.sha256) kubectl" | sha256sum -c \
  && install -o root -g root kubectl /usr/local/bin/kubectl

###########################################################################
# kubeval Stage
###########################################################################
FROM node:$NODE_VERSION as kubeval
ARG KUBEVAL_VERSION=0.16.1

WORKDIR /tmp
RUN curl -LO https://github.com/instrumenta/kubeval/releases/download/v$KUBEVAL_VERSION/kubeval-linux-amd64.tar.gz \
  && curl -LO https://github.com/instrumenta/kubeval/releases/download/v$KUBEVAL_VERSION/checksums.txt \
  && grep kubeval-linux-amd64.tar.gz$ checksums.txt | sha256sum -c \
  && tar xf kubeval-linux-amd64.tar.gz \
  && install -o root -g root kubeval /usr/local/bin/kubeval

###########################################################################
# deps Stage
###########################################################################
FROM node:$NODE_VERSION as deps
RUN apt-get update && apt-get install -y \
    curl \
    netcat-openbsd
# Install Azure CLI `az`
# Reference: https://aka.ms/InstallAzureCli
RUN curl -sL https://aka.ms/InstallAzureCLIDeb | bash
# Install Docker engine `docker`
# Reference: https://docs.docker.com/engine/install/debian/
RUN apt-get install -y apt-transport-https ca-certificates curl gnupg lsb-release \
  && curl -fsSL https://download.docker.com/linux/debian/gpg \
    | gpg --dearmor -o /usr/share/keyrings/docker-archive-keyring.gpg \
  && echo "deb [arch=amd64 signed-by=/usr/share/keyrings/docker-archive-keyring.gpg] https://download.docker.com/linux/debian $(lsb_release -cs) stable" \
      | tee /etc/apt/sources.list.d/docker.list > /dev/null \
  && apt-get update \
  && apt-get install -y docker-ce docker-ce-cli containerd.io supervisor \
  && usermod --append --groups docker node
COPY components/si-veritech/supervisor/dockerd.conf /etc/supervisor/conf.d/
COPY components/si-veritech/supervisor/entrypoint.sh /
# Install AWS CLI `aws`
COPY --from=awscli /usr/local/aws-cli/ /usr/local/aws-cli/
COPY --from=awscli /usr/local/aws-cli-bin/ /usr/local/bin/
# Install kubectl
COPY --from=kubectl /usr/local/bin/kubectl /usr/local/bin/kubectl
# Install kubeval
COPY --from=kubeval /usr/local/bin/kubeval /usr/local/bin/kubeval

###########################################################################
# Final Stage
###########################################################################
FROM deps as final

RUN apt-get update && apt-get install -y make
WORKDIR /usr/src
COPY --chown=node:node . .

USER node

RUN cd components/si-registry && make build
RUN cd components/si-entity && make build
RUN cd components/si-veritech && make build

WORKDIR components/si-veritech

USER root

EXPOSE 5157/tcp
ENTRYPOINT [ \
  "/bin/sh", "/entrypoint.sh", "--", \
    "/sbin/runuser", "-u", "node", "--", \
      "/usr/bin/make", "watch" \
]
