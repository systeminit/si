# Terraform

## Build base system

```console
host$ docker build -t research - <Dockefile
```

## Enter base system

```console
host$ docker run -ti -v "$(pwd):/home/bobo/src" research
```

You should be sitting at a Bash shell prompt as user `bobo` on a fresh Arch
Linux system with [Paru](https://github.com/morganamilo/paru) installed for AUR
packages and passwordless `sudoers` privileges.

## Install Terraform

```sh
sudo pacman -S --noconfirm terraform
terraform -install-autocomplete && . ~/.bashrc
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
terraform init
```

```sh
terraform plan
terraform apply
```

The URL for the load balancer-exposed Whiskers service should be (eventually)
accessible with the `whiskers_url` Terraform output.

## Teardown

```sh
terraform destroy
rm -rf .terraform .terraform.lock.hcl terraform.tfstate*
exit
```
