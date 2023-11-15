#!/bin/bash

# vars
GITROOT="$(git rev-parse --show-toplevel)"
PACKAGEDIR="$GITROOT/cyclone-pkg"
ROOTFS="$PACKAGEDIR/cyclone-rootfs.ext4"
ROOTFSMOUNT="$PACKAGEDIR/rootfs"
GUESTDISK="/rootfs"
INITSCRIPT="$PACKAGEDIR/init.sh"

# create disk and mount to a known locations
sudo rm -rf $PACKAGEDIR
mkdir -p $ROOTFSMOUNT $KERNELMOUNT
dd if=/dev/zero of=$ROOTFS bs=1M count=1024
mkfs.ext4 $ROOTFS
sudo mount $ROOTFS $ROOTFSMOUNT

# create our script to add an init system to our container image
cat << EOL > $INITSCRIPT
apk add openrc mingetty

# Make sure special file systems are mounted on boot:
rc-update add devfs boot
rc-update add procfs boot
rc-update add sysfs boot
rc-update add local boot

# Then, copy the newly configured system to the rootfs image:
for d in bin etc lib root sbin usr nix; do tar c "/\${d}" | tar x -C ${GUESTDISK}; done
for dir in dev proc run sys var; do mkdir ${GUESTDISK}/\${dir}; done

# autologin
echo "ttyS0::respawn:/sbin/mingetty --autologin root --noclear ttyS0" >> ${GUESTDISK}/etc/inittab
sed -i 's/root:*::0:::::/root:::0:::::/g' $GUESTDISK/etc/shadow

# autostart cyclone
cat << EOF > /rootfs/etc/init.d/cyclone
#!/sbin/openrc-run

name="cyclone"
description="Cyclone"
supervisor="supervise-daemon"
command="cyclone"
command_args="--bind-vsock 3:52 --decryption-key /dev.decryption.key --lang-server /usr/local/bin/lang-js --enable-watch --limit-requests 1 --watch-timeout 10 --enable-ping --enable-resolver --enable-action-run -v"
pidfile="/run/agent.pid"
EOF

chmod +x ${GUESTDISK}/usr/local/bin/cyclone
chmod +x ${GUESTDISK}/usr/local/bin/lang-js
chmod +x ${GUESTDISK}/etc/init.d/cyclone

chroot ${GUESTDISK} rc-update add cyclone boot

EOL

# run the script, mounting the disk so we can create a rootfs
sudo docker run \
  -v $ROOTFSMOUNT:$GUESTDISK \
  -v $INITSCRIPT:/init.sh \
  -it --rm \
  --entrypoint sh \
  systeminit/cyclone:sha-880fc9c75cccf0159ba20129f02c9f559301bb56-dirty-amd64 \
  /init.sh

# lets go find the dev decryption key for now
sudo cp $GITROOT/lib/cyclone-server/src/dev.decryption.key $ROOTFSMOUNT

# cleanup the PACKAGEDIR
sudo umount $ROOTFSMOUNT
rm -rf $ROOTFSMOUNT $KERNELMOUNT $INITSCRIPT $KERNELISO

# make the package
#sudo tar -czvf cyclone-package.tar.gz -C $PACKAGEDIR .

sudo mv cyclone-pkg/cyclone-rootfs.ext4 /firecracker-data/rootfs.ext4

# cleanup
#sudo rm -rf $PACKAGEDIR
