#!/bin/bash

set -eou pipefail

check_params_set(){

   if ! test -f ${VARIABLES_FILE:-/tmp/variables.txt}; then
     echo "Error: Could not find var file to drive installation, creating with defaults"
    cat <<EOF > /tmp/variables.txt
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

    test -f /etc/os-release || (echo "Error: Could not find an /etc/os-release file to determine Operating System" && exit 1); [ "$?" -eq 1 ] && exit 1
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
    [[ "$(cat /etc/os-release | grep ^NAME | grep Fedora)" ]] && export OS_VARIANT=fedora && return 0
    [[ "$(cat /etc/os-release | grep ^NAME | grep Ubuntu)" ]] && export OS_VARIANT=ubuntu && return 0
    [[ "$(cat /etc/os-release | grep ^NAME | grep -i pop!_os )" ]] && export OS_VARIANT=ubuntu && return 0
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
        ubuntu)
            # Insert OS specific setup steps here
            echo "Info: executing prereq steps for ubuntu"
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

        # TODO(johnrwatson): Set up cgroup and cpu time/memory limits for jailer.
        # < limits here > - will do later

        # Update Process Limits
        if grep -Fxq "jailer-shared" /etc/security/limits.conf; then
          echo -e "\jailer-shared soft nproc 16384\jailer-shared hard nproc 16384\n" | sudo tee -a /etc/security/limits.conf
        fi

        # Mount secondary EBS volume at /data for
        mkdir -p /firecracker-data/output/ && cd /firecracker-data/

        # Helper Scripts
        curl https://raw.githubusercontent.com/systeminit/si/${CONFIGURATION_MANAGEMENT_BRANCH:-main}/bin/veritech/scripts/start.sh > ./start.sh
        curl https://raw.githubusercontent.com/systeminit/si/${CONFIGURATION_MANAGEMENT_BRANCH:-main}/bin/veritech/scripts/stop.sh > ./stop.sh
        curl https://raw.githubusercontent.com/systeminit/si/${CONFIGURATION_MANAGEMENT_BRANCH:-main}/bin/veritech/scripts/prepare_jailer.sh > ./prepare_jailer.sh

        # Remainder of the binaries
        # TODO(scott): perform some kind of check to decide if we should
        # download these or not to avoid long downloads if we can.
        if $DOWNLOAD_ROOTFS; then
          wget https://artifacts.systeminit.com/cyclone/stable/rootfs/linux/$(uname -m)/cyclone-stable-rootfs-linux-$(uname -m).ext4 -O ./rootfs.ext4
        fi

        if $DOWNLOAD_KERNEL; then
          wget https://si-tools-prod-ec2-firecracker-config.s3.amazonaws.com/firecracker/latest/image-kernel.bin -O ./image-kernel.bin
        fi

        wget https://si-tools-prod-ec2-firecracker-config.s3.amazonaws.com/firecracker/latest/firecracker -O ./firecracker
        wget https://si-tools-prod-ec2-firecracker-config.s3.amazonaws.com/firecracker/latest/jailer -O ./jailer

        # Create a device mapped to the rootfs file of the size of the file.
        # This lets us then create another device that is that size plus 5gb
        # in order to create a copy-on-write layer. This is so we can avoid
        # copying this rootfs around to each jail.
        if ! dmsetup info rootfs &> /dev/null; then
          BASE_LOOP=$(losetup --find --show --read-only ./rootfs.ext4)
          OVERLAY_FILE=./rootfs-overlay
          touch $OVERLAY_FILE
          truncate --size=5368709120 $OVERLAY_FILE
          OVERLAY_LOOP=$(losetup --find --show $OVERLAY_FILE)
          BASE_SZ=$(blockdev --getsz $BASE_LOOP)
          OVERLAY_SZ=$(blockdev --getsz $OVERLAY_LOOP)
          printf "0 $BASE_SZ linear $BASE_LOOP 0\n$BASE_SZ $OVERLAY_SZ zero"  | dmsetup create rootfs
          echo "0 $OVERLAY_SZ snapshot /dev/mapper/rootfs $OVERLAY_LOOP P 8" | dmsetup create rootfs-overlay
        fi

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

        # Create a user and group to run firecracker/jailer with & another group for the shared folders
        if ! id jailer-shared >/dev/null 2>&1; then
          useradd -M jailer-shared
          usermod -L jailer-shared
          groupadd -g 10000 jailer-processes
          usermod -a -G jailer-processes jailer-shared
        fi

        # Set up correct permissions for the /firecracker-data/ folder
        chown -R jailer-shared:jailer-shared /firecracker-data/
        chmod a+x /firecracker-data/*{.sh,firecracker,jailer}
        # chmod 400 /firecracker-data/micro-vm-key

        # Copy bins to /usr/bin/
        cp ./firecracker /usr/bin/firecracker
        cp ./jailer /usr/bin/jailer

        # Load kernel module
        modprobe kvm_intel || echo "loading AMD instead" || modprobe kvm_amd

        # TODO(johnrwatson): Can do better than this, needs review
        chmod 777 /dev/kvm

        # Configure packet forwarding
        sysctl -w net.ipv4.conf.all.forwarding=1

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
        # Block calls to AWS Metadata not coming from the primary network
        if ! iptables -C FORWARD -d 169.254.169.254 -j DROP; then
          iptables -A FORWARD -d 169.254.169.254 -j DROP
        fi

        # Adjust MTU to make it consistent
        ip link set dev $(ip route get 8.8.8.8 | awk -- '{printf $5}') mtu 1500

        # This permits NAT from within the Jail to access the otelcol running on the external interface of the machine. Localhost is `not` resolveable from
        # within the jail or the micro-vm directly due to /etc/hosts misalignment. Hardcoding the destination to 12.0.0.1 for the otel endpoint allows us to
        # ship a static copy of the rootfs but allow us to keep the dynamic nature of the machine hosting. 
        if ! iptables -t nat -C PREROUTING -p tcp --dport 4316 -d 1.0.0.1 -j DNAT --to-destination $(ip route get 8.8.8.8 | awk -- '{printf $7}'):4317; then
          iptables -t nat -A PREROUTING -p tcp --dport 4316 -d 1.0.0.1 -j DNAT --to-destination $(ip route get 8.8.8.8 | awk -- '{printf $7}'):4317
        fi

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

prepare_jailers() {
  ITERATIONS="${1:-100}" # Default to 100 jails
  DOWNLOAD_ROOTFS="${2:-false}" # Default to false
  DOWNLOAD_KERNEL="${3:-false}" # Default to false
  FORCE_CLEAN_JAILS="${4:-false}" # Default to false

  # we need to recreate jails if we get a new kernel or rootfs.
  # This is heavy-handed and should be mnade more specific.
  if $DOWNLOAD_ROOTFS || $DOWNLOAD_KERNEL || $FORCE_CLEAN_JAILS; then
    echo "Force cleaning jails due to passed flags..."
    IN_PARALLEL=1
    SECONDS=0
    for (( iter=0; iter<$ITERATIONS; iter++ ))
    do
      echo -ne "Cleaning jail $(($iter + 1 )) out of $ITERATIONS ... \r"
        # this ensures we only run n jobs in parallel at a time to avoid
        # process locks. This is an unreliable hack.
        if [ $(jobs -r | wc -l) -ge $IN_PARALLEL ]; then
         wait $(jobs -r -p | head -1)
        fi
        /firecracker-data/stop.sh $iter &> /dev/null &
    done
    echo
    echo "Elapsed: $(($SECONDS / 3600))hrs $((($SECONDS / 60) % 60))min $(($SECONDS % 60))sec"
  fi

  if test -f "/firecracker-data/prepare_jailer.sh"; then
    IN_PARALLEL=1
    SECONDS=0
    for (( iter=0; iter<$ITERATIONS; iter++ ))
    do
      echo -ne "Validating jail $(($iter + 1 )) out of $ITERATIONS ... \r"
        # this ensures we only run n jobs in parallel at a time to avoid
        # process locks. This is an unreliable hack.
        # TODO(scott): we need to walk through the processes called in this script
        # and understand where locking could occur. Parallelization can be
        # dangerous here, but testing implies that it works.
        if [ $(jobs -r | wc -l) -ge $IN_PARALLEL ]; then
         wait $(jobs -r -p | head -1)
        fi
        /firecracker-data/prepare_jailer.sh $iter &
    done
    echo
    echo "Elapsed: $(($SECONDS / 3600))hrs $((($SECONDS / 60) % 60))min $(($SECONDS % 60))sec"
  else
    echo "prepare_jailer.sh script not found, skipping jail creation."
    exit 1
  fi
}

# -----------------------------------------
usage() {
   echo "Usage: $0 [ -v /tmp/variables.txt ] [ -j 1000 ] [ -r ] [ -k ] [ -c ]" 1>&2
   echo
   echo "Examples:"
   echo "$0 -v /tmp/variables.txt -j 100 -rk " 1>&2
   echo "Download a new rootfs and kernel and then create 100 new jails."
   echo
   echo "$0 -v /tmp/variables.txt -j 10 -c" 1>&2
   echo "Force clean the existing jails and then create 10 new ones."
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
   echo "       This will force a recreation of all jails."
   echo "-c     Force clean the jails to recreate all of them."
   echo
}

DOWNLOAD_ROOTFS=false
DOWNLOAD_KERNEL=false
JAILS_TO_CREATE=100
FORCE_CLEAN_JAILS=false

while getopts "hv:j:rkc" flag;
do
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
   c) #  Whether to force clean created jails
   FORCE_CLEAN_JAILS=true
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
prepare_jailers $JAILS_TO_CREATE $DOWNLOAD_ROOTFS $DOWNLOAD_KERNEL $FORCE_CLEAN_JAILS
execute_cleanup
