# Helm

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

## Install Helm

```sh
sudo pacman -S --noconfirm helm
echo 'eval "$(helm completion bash)"' >>~/.bashrc && . ~/.bashrc
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
sudo pacman -S --noconfirm kubectl bash-completion
bash # required to re-load bash-completion helpers
echo 'eval "$(kubectl completion bash)"' >>~/.bashrc && . ~/.bashrc
aws eks update-kubeconfig --region us-east-2 --name democluster
```

Test connectivity with:

```sh
kubectl get services
```

## Create Helm Chart

```sh
cd src
helm create whiskers
```

The auto-generated template is a little on the overwhelm-you-with-complexity but
changing a few variables in `values.yaml` gets us to a deployment which works
and is reasonably close to our ideal (essentially with extra metadata/labels).

## Deploy

```sh
cd ~/src
helm install -n helmet --create-namespace whiskers-are-we ./whiskers
```

In the notes section (output at the end of the command) there is a 2-line series
of shell commands which will output the AWS load balancer address.

## Teardown

```sh
helm uninstall -n helmet whiskers-are-we
kubectl delete namespaces helmet
exit
exit
```
