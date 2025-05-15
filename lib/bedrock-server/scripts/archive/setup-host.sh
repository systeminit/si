#!/bin/bash

export SI_SERVICE=rebaser
export SI_VERSION="stable"

# install build
wget https://artifacts.systeminit.com/${SI_SERVICE}/stable/omnibus/linux/$(arch)/$SI_SERVICE-stable-omnibus-linux-$(arch).tar.gz -O - | tar -xzvf - -C /

# prep system
mkdir -p /run/app
wget https://raw.githubusercontent.com/systeminit/si/${BRANCH:-main}/component/deploy/docker-compose.yaml -O /run/app/docker-compose.yaml


# Configuring the machine to use buck2
curl -L https://nixos.org/nix/install | sh -s -- --daemon
source ~/.bashrc

DIR_ENV_VERSION=2.36.0
curl -LO https://github.com/direnv/direnv/releases/download/v${DIR_ENV_VERSION}/direnv.linux-amd64
chmod +x direnv.linux-amd64
sudo mv direnv.linux-amd64 /usr/local/bin/direnv
direnv --version
eval "$(direnv hook bash)"

yum install git -y

git clone https://github.com/systeminit/si.git
direnv allow ./si
cd si

DEV_HOST=0.0.0.0 TILT_HOST=0.0.0.0 buck2 run dev:up