#!/bin/bash

set -eou pipefail

check_params_set() {

  if ! test -f ${VARIABLES_FILE:-/tmp/variables.txt}; then
    echo "Error: Could not find var file to drive installation, creating with defaults"
    cat <<EOF >/tmp/variables.txt
      CONFIGURATION_MANAGEMENT_TOOL="shell"
      CONFIGURATION_MANAGEMENT_BRANCH="main"
      AUTOMATED="true"
EOF
  fi

  echo "---------------------------------"
  echo "Values passed as inputs:"
  echo "VARIABLES_FILE=${VARIABLES_FILE:-/tmp/variables.txt}"
  cat ${VARIABLES_FILE:-/tmp/variables.txt}
  eval $(cat ${VARIABLES_FILE:-/tmp/variables.txt})
  echo "DOWNLOAD_ROOTFS=$DOWNLOAD_ROOTFS"
  echo "DOWNLOAD_KERNEL=$DOWNLOAD_KERNEL"
  echo "JAILS_TO_CREATE=$JAILS_TO_CREATE"
  echo "FORCE_CLEAN_JAILS=$FORCE_CLEAN_JAILS"
  echo "---------------------------------"

  [[ "$AUTOMATED" != "true" ]] && sleep 5 # Giving some time for real users to review the vars file

}

check_os_release() {

  test -f /etc/os-release || (echo "Error: Could not find an /etc/os-release file to determine Operating System" && exit 1)
  [ "$?" -eq 1 ] && exit 1
  echo "------------------------------------"
  echo "Info: /etc/os-release shown below:"
  cat /etc/os-release
  echo "------------------------------------"
  [[ "$(cat /etc/os-release | grep 'CentOS Linux release 7')" ]] && export OS_VARIANT=centos-7 && return 0
  [[ "$(cat /etc/os-release | grep 'CentOS Stream release 8')" ]] && export OS_VARIANT=centos-stream-8 && return 0
  [[ "$(cat /etc/os-release | grep 'Rocky Linux release 8')" ]] && export OS_VARIANT=rocky-8 && return 0
  [[ "$(cat /etc/os-release | grep 'Red Hat Enterprise Linux Server release 7')" ]] && export OS_VARIANT=redhat-7 && return 0
  [[ "$(cat /etc/os-release | grep 'Red Hat Enterprise Linux release 8')" ]] && export OS_VARIANT=redhat-8 && return 0
  [[ "$(cat /etc/os-release | grep 'Amazon Linux release 2')" ]] && export OS_VARIANT=amazon-linux-2 && return 0
  [[ "$(cat /etc/os-release | grep 'Amazon Linux 2023')" ]] && export OS_VARIANT=amazon-linux-2023 && return 0
  [[ "$(cat /etc/os-release | grep 'Arch Linux')" ]] && export OS_VARIANT=arch-linux && return 0
  [[ "$(cat /etc/os-release | grep ^NAME | grep Fedora)" ]] && export OS_VARIANT=fedora && return 0
  [[ "$(cat /etc/os-release | grep ^NAME | grep Ubuntu)" ]] && export OS_VARIANT=ubuntu && return 0
  [[ "$(cat /etc/os-release | grep ^NAME | grep -i pop!_os)" ]] && export OS_VARIANT=ubuntu && return 0
  [[ "$(cat /etc/os-release | grep ^NAME | grep Debian)" ]] && export OS_VARIANT=debian && return 0
  [[ "$(cat /etc/os-release | grep ^NAME | grep Mint)" ]] && export OS_VARIANT=mint && return 0

  echo "Error: Operating system could not be determined or is unsupported, could not configure the OS for firecracker node" && exit 1

}

install_pre_reqs() {

  echo "Info: Installing prereqs for configuration"

  case $OS_VARIANT in
    centos-7)
      # Insert OS specific setup steps here
      sudo yum -v update -y
      ;;
    redhat-7)
      # Insert OS specific setup steps here
      sudo yum -v update -y
      ;;
    centos-stream-8)
      # Insert OS specific setup steps here
      sudo yum -v update -y
      ;;
    redhat-8)
      # Insert OS specific setup steps here
      sudo yum -v update -y
      ;;
    rocky-8)
      # Insert OS specific setup steps here
      sudo yum -v update -y
      ;;
    amazon-linux-2)
      # Insert OS specific setup steps here
      sudo yum -v update -y
      ;;
    amazon-linux-2023)
      # Insert OS specific setup steps here
      sudo yum -v update -y
      ;;
    ubuntu)
      # Insert OS specific setup steps here
      echo "Info: executing prereq steps for ubuntu"
      ;;
    arch-linux)
      echo "Info: executing prereq steps for arch linux"
      ;;
    *)
      echo "Error: Something went wrong, OS_VARIANT determined to be: $OS_VARIANT (unsupported)" && exit 1
      ;;

  esac

  [[ $? != 0 ]] && echo "Error: Exit code $? returned during installation; see above error log for information"

  return 0

}
execute_configuration_management() {

  echo "Info: Installation folder set to /firecracker-data/"

  if [[ $CONFIGURATION_MANAGEMENT_TOOL == "shell" ]]; then
    DOWNLOAD_ROOTFS="${1:-false}" # Default to false
    DOWNLOAD_KERNEL="${2:-false}" # Default to false

    # Clear up any jails that may well be lingering, sometimes it happens
    # when jails are left behind and it causes grief with device usage
    # on startup after upgrade in place or similar.
    pkill -e -f '^/firecracker --id' || true

    # Update Process Limits
    if grep -Fxq "jailer-shared" /etc/security/limits.conf; then
      echo -e "\jailer-shared soft nproc 16384\jailer-shared hard nproc 16384\n" | sudo tee -a /etc/security/limits.conf
    fi

    # Mount secondary EBS volume at /data for
    mkdir -p /firecracker-data/output/ && cd /firecracker-data/

    arch=$(uname -m)
    # Remainder of the binaries
    if $DOWNLOAD_ROOTFS; then
      url="https://artifacts.systeminit.com/cyclone/stable/rootfs/linux/${arch}/cyclone-stable-rootfs-linux-${arch}.ext4"
      # we keep track of the version, so if this is either the first time we
      # are downloading or if the version is the same we should be good to
      # skip pulling this again.
      rootfs_version_file="./rootfs_version"
      rootfs_version=$(wget -qO- $url.metadata.json | jq -r '.version')

      if [ ! -f "$rootfs_version_file" ] || ! grep -q "^$rootfs_version$" "$rootfs_version_file"; then
        echo "Pulling version ${rootfs_version}"
        echo $rootfs_version >$rootfs_version_file
        wget $url -O ./rootfs.ext4

        # We have a new rootfs so we must create a new overlay.
        # Create a device mapped to the rootfs file of the size of the file.
        # This lets us then create another device that is that size plus 5gb
        # in order to create a copy-on-write layer. This is so we can avoid
        # copying this rootfs around to each jail.
        if dmsetup info rootfs &>/dev/null; then
          losetup -D
          dmsetup remove rootfs-overlay || true
          dmsetup remove rootfs || true
        fi
        BASE_LOOP=$(losetup --find --show --read-only ./rootfs.ext4)
        OVERLAY_FILE=./rootfs-overlay
        touch $OVERLAY_FILE
        truncate --size=10737418240 $OVERLAY_FILE
        OVERLAY_LOOP=$(losetup --find --show $OVERLAY_FILE)
        BASE_SZ=$(blockdev --getsz $BASE_LOOP)
        OVERLAY_SZ=$(blockdev --getsz $OVERLAY_LOOP)
        printf "0 $BASE_SZ linear $BASE_LOOP 0\n$BASE_SZ $OVERLAY_SZ zero" | dmsetup create rootfs
        echo "0 $OVERLAY_SZ snapshot /dev/mapper/rootfs $OVERLAY_LOOP P 8" | dmsetup create rootfs-overlay
      fi
    fi

    if $DOWNLOAD_KERNEL; then
      wget https://artifacts.systeminit.com/firecracker/latest/${arch}/image-kernel.bin -O ./image-kernel.bin
    fi

    # NOTE: Commented out - using locally installed firecracker/jailer instead of S3 artifacts
    # wget https://artifacts.systeminit.com/firecracker/latest/${arch}/firecracker -O ./firecracker
    # wget https://artifacts.systeminit.com/firecracker/latest/${arch}/jailer -O ./jailer

    # TODO(scott): fix me.
    # This will perform the same CoW layering for the kernel. First pass
    # here caused issues. The kernel image is only 27mb, so I'm leaving
    # this commented out until we decide we need that space back.
    # if ! dmsetup info kernel; then
    #   BASE_LOOP=$(losetup --find --show --read-only ./image-kernel.bin)
    #   OVERLAY_FILE=./kernel-overlay
    #   touch $OVERLAY_FILE
    #   truncate --size=27721848 $OVERLAY_FILE
    #   OVERLAY_LOOP=$(losetup --find --show $OVERLAY_FILE)
    #   BASE_SZ=$(blockdev --getsz $BASE_LOOP)
    #   OVERLAY_SZ=$(blockdev --getsz $OVERLAY_LOOP)
    #   printf "0 $BASE_SZ linear $BASE_LOOP 0\n$BASE_SZ $OVERLAY_SZ zero"  | dmsetup create kernel
    #   echo "0 $OVERLAY_SZ snapshot /dev/mapper/kernel $OVERLAY_LOOP P 8" | dmsetup create kernel-overlay
    # fi

    # TODO(johnrwatson): Currently not used but we could maybe make dynamic keys for each micro-vm (or use something like aws ssm/tailscale)
    # This is a bit of a poor attempt to setup a child key, but will do until we have this properly working
    # if [[ -z "$FIRECRACKER_SSH_KEY" ]]; then
    #   ssh-keygen -b 2048 -t rsa -f /firecracker-data/micro-vm-key -q -N ""
    # else
    #   mv $FIRECRACKER_SSH_KEY /firecracker-data/micro-vm-key -f
    # fi

    # Ensure we set the UID max to greater than our desired range
    sudo sed -i 's/^UID_MAX\t\t*[0-9]\+$/UID_MAX\t\t600000/' /etc/login.defs

    # Create a user and group to run firecracker/jailer with & another group for the shared folders
    if ! id jailer-shared >/dev/null 2>&1; then
      useradd -M -u 40000 jailer-shared
      usermod -L jailer-shared
      groupadd -g 10000 jailer-processes
      usermod -a -G jailer-processes jailer-shared
    fi

    # Set up correct permissions for the /firecracker-data/ folder
    chown -R jailer-shared:jailer-shared /firecracker-data/
    # NOTE: Commented out - using locally installed firecracker/jailer
    # chmod a+x /firecracker-data/{firecracker,jailer}
    # chmod 400 /firecracker-data/micro-vm-key

    # Copy bins to /usr/bin/
    # NOTE: Commented out - using locally installed firecracker/jailer
    # cp ./firecracker /usr/bin/firecracker
    # cp ./jailer /usr/bin/jailer

    # Load kernel module
    modprobe kvm_intel || echo "loading AMD instead" || modprobe kvm_amd

    # TODO(johnrwatson): Can do better than this, needs review
    chmod 777 /dev/kvm

    # Configure packet forwarding
    sysctl -w net.ipv4.conf.all.forwarding=1

    # Configure TCP memory and write buffer limits
    # Previously: net.ipv4.tcp_mem = 3079470 4105960 6158940
    sysctl -w net.ipv4.tcp_mem="8388608 12582912 16777216"
    # Previously: net.ipv4.tcp_wmem = 4096 20480 4194304
    sysctl -w net.ipv4.tcp_wmem="4096 524288 16777216"
    # Previously: net.ipv4.tcp_rmem = 4096 131072 6291456
    sysctl -w net.ipv4.tcp_rmem="4096 524288 16777216"

    # Optimize TCP keepalive settings
    # Previously: net.ipv4.tcp_keepalive_time = 7200
    sysctl -w net.ipv4.tcp_keepalive_time=30 # Detect stale connections faster
    # Previously: net.ipv4.tcp_keepalive_intvl = 75
    sysctl -w net.ipv4.tcp_keepalive_intvl=10 # Reduce keepalive interval
    # Previously: net.ipv4.tcp_keepalive_probes = 9
    sysctl -w net.ipv4.tcp_keepalive_probes=5 # Reduce keepalive retries

    # Prevent SYN drops under high load
    # Previously: net.ipv4.tcp_max_syn_backlog = 4096
    sysctl -w net.ipv4.tcp_max_syn_backlog=8192

    # Allow more queued connections
    # Previously: net.core.somaxconn = 4096
    sysctl -w net.core.somaxconn=8192

    # Reduce stale socket overhead
    # net.ipv4.tcp_fin_timeout = 60
    sysctl -w net.ipv4.tcp_fin_timeout=15

    # Reuse closed connections faster
    # Previously net.ipv4.tcp_tw_reuse = 2
    sysctl -w net.ipv4.tcp_tw_reuse=1

    # -----------------------------------------------------
    # ------ Cgroup Allocation ----------------------------
    # -----------------------------------------------------

    # Define Cgroup Names
    VERITECH_CGROUP="/sys/fs/cgroup/veritech"
    FIRECRACKER_CGROUP="/sys/fs/cgroup/veritech/firecracker"

    # Create Cgroup for veritech
    mkdir -p "$VERITECH_CGROUP"

    # Marks Veritech as a CPU partition root so it can further delegate cpu partitions
    echo "root" >"$VERITECH_CGROUP/cpuset.cpus.partition"

    # Assign CPU affinity for the parent cgroup, i.e. all cores
    echo "0-63" >"$VERITECH_CGROUP/cpuset.cpus"

    # Ensure memory comes from NUMA node 0 (needed for cpu affinity)
    echo "0" >"$VERITECH_CGROUP/cpuset.mems"

    # Allow veritech to delegate cpuset
    echo "+cpuset" >/sys/fs/cgroup/veritech/cgroup.subtree_control

    # Create Firecracker’s parent cgroup under Veritech
    mkdir -p "$FIRECRACKER_CGROUP"

    # Marks Firecracker as a normal cgroup
    echo "member" >"$FIRECRACKER_CGROUP/cpuset.cpus.partition"

    # Restrict Firecracker instances to CPUs 8-63
    echo "8-63" >"$FIRECRACKER_CGROUP/cpuset.cpus"

    # Ensure memory comes from NUMA node 0 (needed for cpu allocation)
    echo "0" >"$FIRECRACKER_CGROUP/cpuset.mems"

    # Get Current Process ID
    VERITECH_PID=$$

    # Check if we found any matching processes
    if [[ -n "$VERITECH_PID" ]]; then
      echo "Assigning Veritech processes to cgroup..."
      for PID in $VERITECH_PID; do
        echo $PID >"$VERITECH_CGROUP/cgroup.procs"
      done
    else
      echo "Warning: Veritech process not found, unable to allocate cgroup!"
    fi

    # -----------------------------------------------------

    # Avoid "nf_conntrack: table full, dropping packet"
    #sudo sysctl -w net.ipv4.netfilter.ls=99999999

    # Avoid "neighbour: arp_cache: neighbor table overflow!"
    sysctl -w net.ipv4.neigh.default.gc_thresh1=1024
    sysctl -w net.ipv4.neigh.default.gc_thresh2=2048
    sysctl -w net.ipv4.neigh.default.gc_thresh3=4096

    # Masquerade all external traffic as if it was wrong the external interface
    if ! iptables -t nat -C POSTROUTING -o $(ip route get 8.8.8.8 | awk -- '{printf $5}') -j MASQUERADE; then
      iptables -t nat -A POSTROUTING -o $(ip route get 8.8.8.8 | awk -- '{printf $5}') -j MASQUERADE
    fi

    # Allow forwarding in the default network namespace to allow NAT'ed traffic leave
    # NB: iptables doesn't support -C for the rule checking of protocols
    iptables -P FORWARD ACCEPT

    # Block calls to AWS Metadata not coming from the primary network
    if ! iptables -C FORWARD -d 169.254.169.254 -j DROP; then
      iptables -A FORWARD -d 169.254.169.254 -j DROP
    fi

    # Adjust MTU to make it consistent
    ip link set dev $(ip route get 8.8.8.8 | awk -- '{printf $5}') mtu 1500

    # Save/Reload Kernel Settings
    sysctl -p

  else
    echo "Error: Unsupported or unknown configuration management tool specified, exiting."
    exit 1
  fi

  echo "Info: System configuration complete"

}

execute_cleanup() {

  case $OS_VARIANT in
    centos-7)
      # Insert OS specific cleanup steps here
      yum -v clean all
      ;;
    redhat-7)
      # Insert OS specific cleanup steps here
      yum -v clean all
      ;;
    centos-stream-8)
      # Insert OS specific cleanup steps here
      yum -v clean all
      ;;
    redhat-8)
      # Insert OS specific cleanup steps here
      yum -v clean all
      ;;
    rocky-8)
      # Insert OS specific cleanup steps here
      yum -v clean all
      ;;
    amazon-linux-2)
      # Insert OS specific cleanup steps here
      yum -v clean all
      ;;
    amazon-linux-2023)
      # Insert OS specific cleanup steps here
      yum -v clean all
      ;;
    arch-linux)
      # Insert OS specific cleanup steps here
      echo "Info: Executing post-clean up for arch"
      ;;

    ubuntu)
      # Insert OS specific setup steps here
      echo "Info: Executing post-clean up for ubuntu"
      ;;
    *)
      echo "Error: Something went wrong during cleanup, OS_VARIANT set to: $OS_VARIANT" && exit 1
      ;;
  esac

  rm -Rf /tmp/firecracker-install/*

}

# -----------------------------------------
usage() {
  echo "Usage: $0 [ -v /tmp/variables.txt ] [ -j 1000 ] [ -r ] [ -k ] [ -c ]" 1>&2
  echo
  echo "Examples:"
  echo "$0 -v /tmp/variables.txt -j 100 -rk " 1>&2
  echo "Download a new rootfs and kernel and then create 100 new jails."
  echo
  echo "Prepares a machine to be used with Firecracker."
  echo
  echo "options:"
  echo "-h     Print this Help."
  echo "-v     The path to the required vars file."
  echo "-j     The number of jails to create. Defaults to 1000."
  echo "-r     Whether to download a new rootfs."
  echo "       This will force a recreation of all jails"
  echo "-k     Whether to download a new kernel."
  echo
}

DOWNLOAD_ROOTFS=false
DOWNLOAD_KERNEL=false
JAILS_TO_CREATE=100
FORCE_CLEAN_JAILS=false

while getopts "hv:j:rkc" flag; do
  case $flag in
    h) #
      usage
      exit 1
      ;;
    v) # Variables File
      # Pass the vars file
      VARIABLES_FILE=$OPTARG
      ;;
    j) # Number of jails to create. Defaults to 5k
      JAILS_TO_CREATE=${OPTARG}
      ;;
    r) #  Whether to download a new rootfs
      DOWNLOAD_ROOTFS=true
      ;;
    k) #  Whether to download a new kernel
      DOWNLOAD_KERNEL=true
      ;;
    \?)
      usage
      exit 1
      ;;
  esac
done

check_params_set && echo -e "Installation Values found to be:\n - $VARIABLES_FILE"
check_os_release && echo -e "Operating System found to be:\n - $OS_VARIANT"
install_pre_reqs
execute_configuration_management $DOWNLOAD_ROOTFS $DOWNLOAD_KERNEL
execute_cleanup
