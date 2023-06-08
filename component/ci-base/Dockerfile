FROM registry.fedoraproject.org/fedora-minimal

RUN microdnf install -y git && microdnf clean all -y
RUN curl --proto '=https' --tlsv1.2 -sSf -L https://install.determinate.systems/nix | sh -s -- install linux --init none --no-confirm

WORKDIR /src
COPY flake.nix flake.nix
COPY flake.lock flake.lock
COPY rust-toolchain rust-toolchain

ENV PATH="${PATH}:/nix/var/nix/profiles/default/bin"
RUN nix develop --command echo poop
RUN rm flake.nix flake.lock rust-toolchain

ENTRYPOINT ["nix", "develop"]