# Setup for our custom runner

## Initial Setup

* It's a single reserved c5d.9xlarge
* Running the fedora AMI ami-0133ad8c5d900ddef
* Using the standard si_key ssh key from 1password
* They are named 'ci-1' and 'ci-2' in the aws console
* Running in a separate VPC from everything else

## Basic Setup

```sh
sudo hostnamectl set-hostname ci-2
sudo dnf -y upgrade
sudo dnf -y install @development-tools nvme-cli lld clang cmake openssl libev libevent jq dnf-plugins-core make git lld skopeo wget butane golang-github-instrumenta-kubeval
sudo dnf config-manager --add-repo https://download.docker.com/linux/fedora/docker-ce.repo
sudo dnf install -y docker-ce docker-ce-cli containerd.io docker-compose
sudo systemctl start docker
sudo systemctl enable docker
```

## Setting up ephemeral storage

```sh
sudo parted /dev/nvme1n1
    (parted) mklabel gpt
    (parted) mkpart primary btrfs 0% 100%
    (parted) quit
sudo mkfs.btrfs /dev/nvme1n1p1
sudo mkdir -p /data
```

Extract the `UUID` line from the mkfs output, and use it to add a line to fstab:

```sh
UUID=f5a67474-0763-4d11-8a93-ad6b67488a00 /data                   btrfs   compress=zstd:1 0 0
```

Mount the drive:

```sh 
sudo mount /data
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
sudo useradd -r grunner
sudo mkdir -p /data/runners
```

Use `vipw` to set the grunner users homedirectory to /data/runner

Add `grunner` and `fedora` to the `docker` group (`vi /etc/group`)

## Installing the runners

Follow the instructions at [github](https://github.com/systeminit/si/settings/actions/runners/new). 

Install the runner to /data/runners/X, where X is the number of the runner. (1, for the first - this number
is the number of runners *on this host*; we may decide to run more than 1!).

Follow the instructions at [github](https://docs.github.com/en/actions/hosting-your-own-runners/configuring-the-self-hosted-runner-application-as-a-service) to configure the self hosted runner as a service.

Edit the systemd unit file it generates to switch the required user to 'grunner' rather than 'fedora'

Make sure you enable the selinux context:

```sh 
cd /data/runners/1
sudo restorecon -r -v /etc/systemd/system/actions.runner.*.service
sudo chcon system_u:object_r:usr_t:s0 runsvc.sh
```

Then enable and start it:

```sh
$ systemctl enable actions.runner.systeminit-si.ci-1
$ systemctl start actions.runner.systeminit-si.ci-1
```

## Setting up the runners as `grunner`

```sh 
sudo su - grunner
```

### Installing rust

```sh 
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### Installing volta

```sh 
curl https://get.volta.sh | bash
volta install node typescript
```

## Troubleshooting

Sometimes, there may be a permissions error seen when runners try to clean themselves up from the previous run.
To fix this manually, `ssh` into a runner and delete the _interior_ `si` directory.
Caution: do not delete the _exterior_ `si` directory.

```sh
# This can technically be ran in one command, but let's be careful and "cd" first.
cd /data/runners/1/_work/si/

# We need to use "sudo" due to the permissions error.
sudo rm -rf ./si
```
