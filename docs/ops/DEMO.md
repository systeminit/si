# Documentation for the demo instance

# Management

The instance is running under `tmux` on `demo-1`. 

# Setup

## EC2

Running on a t3.large with the latest fedora AMI.

## Update

```
$ dnf update 
```

## Change hostname

```
$ hostnamectl set-hostname demo-1
```

## Install tailscale

## Install development tools

```
sudo dnf groupinstall "Development Tools" "Development Libraries" g++
```

## Install docker

```
$ dnf install docker
$ systemctl enable docker
$ systemctl start docker
```

Add fedora to the docker group

```
$ vi /etc/group
docker:x:992:fedora
```

## Install nodejs and tmux

```
$ dnf install nodejs tmux
```

## Run bootstrap

```
$ make bootstrap
```

## Run make build

```
$ make build
```

## Start Containers

```
$ cd ./components/postgres/bin/run-container.sh
$ cd ./components/nats/bin/run-container.sh
```

## Start tmux and services

```
$ cd ~/si
$ tmux
```

Hit `ctrl-b+c` twice. There should then be 3 "windows" at the bottom of the tmux
screen. Hit `ctrl-b+0`, and type `cd ~/si/components/si-veritech && make run`.
Then `ctrl-b+1`, and type `cd ~/si/components/si-sdf && make run`. Then Hit
`ctrl-b+2`, and type `cd ~/si/components/si-web-app && make run`.


