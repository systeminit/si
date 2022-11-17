# Staging Host Deployment Documentation

The files in this folder allow you to deploy an EC2
instance that automatically deploy the latest versions
of SI's containers, resetting the database on every update.

Right now, it's only bringing up a coreos instance with  
SI's containers on startup, but no auto-update via watchtower at first boot.

It can be started by, while on the folder containing this file,
running:

```shell
butane staging-1.yaml --pretty --strict --files-dir ../../ > staging-1.ign
terraform apply  -auto-approve
```

for watchtower (a.k.a. auto updates) to work, you need to log in to the server and execute the following to disable
SELinux:

```shell
sudo sed -i -e 's/SELINUX=/SELINUX=disabled #/g' /etc/selinux/config && sudo systemctl reboot
```

The server will reboot and restart all services with auto updates enabled.

The way it's working right now, butane copies the deployment
docker compose files and makefile onto the server,
and executes it. The idea would be to, in the future,
execute each server via its own systemd unit.