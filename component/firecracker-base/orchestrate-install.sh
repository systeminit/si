#!/bin/bash

# Call this function with:
# ./orchestrate-install.sh <filepath to variables>

set -eo pipefail

check_params_set(){

    test -f ${VARIABLES_FILE:-/tmp/variables.txt} || (echo "Error: Could not find VARIABLES_FILE: $VARIABLES_FILE file to drive installation" && exit 1); [ "$?" -eq 1 ] && exit 1

    echo "---------------------------------"
    echo "Values passed as inputs:"
    echo "VARIABLES_FILE=${VARIABLES_FILE:-/tmp/variables.txt}"
    cat ${VARIABLES_FILE:-/tmp/variables.txt}
    eval $(cat ${VARIABLES_FILE:-/tmp/variables.txt})
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

        # TODO(johnrwatson): Set up cgroup and cpu time/memory limits for jailer.
        # < limits here > - will do later

        # Update Process Limits
        if ! grep -Fxq "jailer-shared" /etc/security/limits.conf; then
          echo -e "\jailer-shared soft nproc 16384\jailer-shared hard nproc 16384\n" | sudo tee -a /etc/security/limits.conf
        fi

        # Mount secondary EBS volume at /data for
        mkdir -p /firecracker-data/output/ && cd /firecracker-data/

        # Helper Scripts
        curl https://raw.githubusercontent.com/systeminit/si/${CONFIGURATION_MANAGEMENT_BRANCH:-main}/component/firecracker-base/start.sh > ./start.sh
        curl https://raw.githubusercontent.com/systeminit/si/${CONFIGURATION_MANAGEMENT_BRANCH:-main}/component/firecracker-base/stop.sh > ./stop.sh
        curl https://raw.githubusercontent.com/systeminit/si/${CONFIGURATION_MANAGEMENT_BRANCH:-main}/component/firecracker-base/prepare_jailer.sh > ./prepare_jailer.sh

        # Remainder of the binaries
        wget -bcq https://si-tools-prod-ec2-firecracker-config.s3.amazonaws.com/firecracker/latest/rootfs.ext4 -O ./rootfs.ext4
        wget -bcq https://si-tools-prod-ec2-firecracker-config.s3.amazonaws.com/firecracker/latest/image-kernel.bin -O ./image-kernel.bin
        wget -bcq https://si-tools-prod-ec2-firecracker-config.s3.amazonaws.com/firecracker/latest/firecracker -O ./firecracker
        wget -bcq https://si-tools-prod-ec2-firecracker-config.s3.amazonaws.com/firecracker/latest/jailer -O ./jailer

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
        chmod 400 /firecracker-data/micro-vm-key

        # Copy bins to /usr/bin/
        cp ./firecracker /usr/bin/firecracker
        cp ./jailer /usr/bin/jailer

        # Load kernel module
        modprobe kvm_intel || echo "loading AMD instead" && modprobe kvm_amd

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
        iptables -t nat -A POSTROUTING -o enp4s0 -j MASQUERADE

        # Adjust MTU to make it consistent
        ip link set dev $(ip route get 8.8.8.8 | awk -- '{printf $5}') mtu 1500

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
  if test -f "./prepare_jailer.sh"; then
    ITERATIONS="${1:-100}" # Default to 100 jails
    echo "Creating $ITERATIONS jails..."
    for (( iter=0; iter<$ITERATIONS; iter++ ))
    do
        ./prepare_jailer.sh $iter &
    done
    wait
  else
    echo "prepare_jailer.sh script not found, skipping jail creation."
  fi
}

# -----------------------------------------

VARIABLES_FILE=$1
JAILS_TO_CREATE=$2

check_params_set && echo -e "Installation Values found to be:\n - $VARIABLES_FILE"
check_os_release && echo -e "Operating System found to be:\n - $OS_VARIANT"
install_pre_reqs
execute_configuration_management
prepare_jailers $JAILS_TO_CREATE
execute_cleanup
