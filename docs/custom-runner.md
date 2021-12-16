# Setup for our custom runner

## Initial Setup

* It's a single reserved c5d.9xlarge
* Running the fedora AMI ami-0133ad8c5d900ddef
* Using the standard si_key ssh key from 1password
* It's named 'ci-1' in the aws console
* It is instance id i-00f760653e6c5bbaa
* Running in a separate VPC from everything else
* ec2-3-145-212-22.us-east-2.compute.amazonaws.com
* 3.145.212.2

## Basic Setup

```sh
$ sudo dnf -y upgrade
$ sudo dnf -y install nvme-cli lld clang cmake openssl libev libevent 
$ sudo dnf -y groupinstall 'Development Tools'
```

## Setting up ephemeral storage

```sh
$ sudo parted /dev/nvme1n1
(parted) mklabel gpt
(parted) mkpart primary btrfs 0% 100%
(parted) quit
$ mkfs.btrfs /dev/nvme1n1p1
$ mkdir -p /data
```

Extract the `UUID` line from the mkfs output, and use it to add a line to fstab:

```sh
UUID=f5a67474-0763-4d11-8a93-ad6b67488a00 /data                   btrfs   compress=zstd:1 0 0
```

Mount the drive:

```sh 
$ mount /data
```

Confirm things are good:

```sh
$ df -h
Filesystem      Size  Used Avail Use% Mounted on
devtmpfs         35G     0   35G   0% /dev
tmpfs            35G     0   35G   0% /dev/shm
tmpfs            14G  708K   14G   1% /run
/dev/nvme0n1p5  500G  807M  499G   1% /
/dev/nvme0n1p5  500G  807M  499G   1% /home
/dev/nvme0n1p2  474M   88M  358M  20% /boot
/dev/nvme0n1p3  100M  9.8M   91M  10% /boot/efi
tmpfs            35G     0   35G   0% /tmp
tmpfs           6.9G     0  6.9G   0% /run/user/1000
/dev/nvme1n1p1  839G  3.7M  837G   1% /data
```

And viola - you're gtg.

## Adding users

```sh 
$ sudo useradd -r grunner
$ sudo mkdir -p /data/runners
```

## Installing the runners

Follow the instructions at [github](https://github.com/systeminit/si/settings/actions/runners/new). 

Install the runner to /data/runner/X, where X is the number of the runner. (1, for the first)

Then install it to systemd, with this file in /etc/systemd/system/actions.runner.systeminit-si.ci-1.service

```sh 
[Unit]
Description=GitHub Actions Runner (systeminit-si.ci-1-1)
After=network.target

[Service]
ExecStart=/data/runners/1/runsvc.sh
User=grunner
WorkingDirectory=/data/runners/1
KillMode=process
KillSignal=SIGTERM
TimeoutStopSec=5min

[Install]
WantedBy=multi-user.target
```

Make sure you enable the selinux context:

```sh 
$ restorecon -r -v /etc/systemd/system/actions.runner.*.service
$ chcon system_u:object_r:usr_t:s0 runsvc.sh
```

Then enable and start it:

```sh
$ systemctl enable actions.runner.systeminit-si.ci-1
$ systemctl start actions.runner.systeminit-si.ci-1
```

