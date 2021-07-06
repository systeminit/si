# Kubernetes Native

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

## Deploy

```sh
cd src
kubectl apply -f 01-namespace.yaml
kubectl -n naynay apply -f 02-deployment.yaml
kubectl -n naynay apply -f 03-service.yaml
```

To get the DNS hostname for the AWS load balancer run:

```sh
kubectl -n naynay get services
```

It will be listed under the `EXTERNAL-IP` column.

## Teardown

```sh
kubectl -n naynay delete -f 03-service.yaml
kubectl -n naynay delete -f 02-deployment.yaml
kubectl delete -f 01-namespace.yaml
exit
exit
```
