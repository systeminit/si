# Pulumi

## Build base system

```console
host$ docker build -t research - <../Dockefile
```

## Enter base system

```console
host$ docker run -ti -v "$(pwd):/home/bobo/src" research
```

You should be sitting at a Bash shell prompt as user `bobo` on a fresh Arch
Linux system with [Paru](https://github.com/morganamilo/paru) installed for AUR
packages and passwordless `sudoers` privileges.

## Install Pulumi

```sh
curl -fsSL https://get.pulumi.com | sh
echo 'eval "$(pulumi gen-completion bash)"' >>~/.bashrc && . ~/.bashrc
sudo pacman -S --noconfirm nodejs npm
```

## Install AWS CLI (v2)

```sh
paru -S --noconfirm aws-cli-v2-bin
echo 'complete -C /usr/bin/aws_completer aws' >>~/.bashrc && . ~/.bashrc
```

```sh
aws configure
```

Enter a value for (all others can be blank/none/default):

- `AWS Access Key ID`
- `AWS Secret Access Key`

## Install kubectl

```sh
sudo pacman -S --noconfirm kubectl
echo 'eval "$(kubectl completion bash)"' >>~/.bashrc && . ~/.bashrc
aws eks update-kubeconfig --region us-east-2 --name democluster
```

Test connectivity with:

```sh
kubectl get services
```

## Create Project

```sh
cd src
mkdir whiskers
cd whiskers
pulumi new kubernetes-typescript -g
```

Ah TypeScript, hello friend! Oh and you brought your editor language servers
with you so that I can tab-complete my way through everything? Even better!

## Deploy

```sh
cd ~/src/whiskers
pulumi login
npm install
pulumi up
```

## Teardown

```sh
pulumi destroy
pulumi stack rm
rm -rf node_modules
exit
```
