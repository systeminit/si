ARG ARCH_VERSION=latest
FROM archlinux:$ARCH_VERSION

ARG ARCH_VERSION
ARG USER=bobo
ARG GROUP=$USER

RUN pacman -Syu --noconfirm \
  && pacman -S --noconfirm bash bash-completion sudo \
  && echo "%wheel ALL=(ALL) NOPASSWD:ALL" > /etc/sudoers.d/01_wheel \
  && groupadd --gid 1000 "$GROUP" \
  && useradd --create-home --shell /bin/bash --groups wheel --gid "$GROUP" --uid 1000 "$USER" \
  && echo "${USER}:${GROUP}" | chpasswd

USER ${USER}:${GROUP}
ENV USER="$USER"
WORKDIR /home/$USER

RUN sudo pacman -S --noconfirm --needed base-devel git \
  && git clone https://aur.archlinux.org/paru.git /tmp/paru \
  && cd /tmp/paru \
  && makepkg -si --noconfirm \
  && cd - \
  && rm -rf /tmp/paru

CMD ["/bin/bash", "-l"]
