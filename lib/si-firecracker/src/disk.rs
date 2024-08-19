use crate::errors::FirecrackerJailError;
use devicemapper::{DevId, DmName, DmOptions, DM};
use krataloopdev::LoopDevice;
use nix::mount::umount;
use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};
use std::{fs::remove_dir_all, result};
use tracing::trace;

type Result<T> = result::Result<T, FirecrackerJailError>;

const JAIL_PATH_PREFIX: &str = "/srv/jailer/firecracker/";
const ROOTFS: &str = "rootfs.ext4";
const OVERLAY_PREFIX: &str = "rootfs-overlay-";

#[derive(Debug)]
pub struct FirecrackerDisk {}

impl FirecrackerDisk {
    pub fn clean(id: u32) -> Result<()> {
        Self::unmount(id)?;
        Self::remove_overlay(id)?;
        Self::remove_loop(id)?;
        Self::remove_jail(id)?;
        Ok(())
    }

    fn unmount(id: u32) -> Result<()> {
        let device = Self::jail_dir_from_id(id).join(ROOTFS);
        if device.exists() {
            trace!("Unmounting device {}", device.display());
            umount(&device)?;
        }
        Ok(())
    }

    fn remove_overlay(id: u32) -> Result<()> {
        let overlay = Self::overlay_from_id(id);
        let dm = DM::new()?;
        let device = DmName::new(&overlay)?;
        let dev_id = &DevId::Name(device);
        if dm.device_info(dev_id).is_ok() {
            dm.device_remove(dev_id, DmOptions::private())?;
        }
        Ok(())
    }

    fn remove_loop(id: u32) -> Result<()> {
        let device = Self::jail_dir_from_id(id).join(Self::overlay_from_id(id));
        if let Some(loopdev) = Self::find_loop_device_by_backing_file(&device)? {
            trace!("Detaching from loop device {}", device.display());
            let device = LoopDevice::open(loopdev)?;
            device.detach()?;
        }
        Ok(())
    }

    fn remove_jail(id: u32) -> Result<()> {
        let jail = &Self::jail_dir_from_id(id);
        if jail.exists() {
            trace!("Removing {}", jail.display());
            remove_dir_all(jail)?;
        }
        Ok(())
    }

    pub fn jail_dir_from_id(id: u32) -> PathBuf {
        let path = PathBuf::from(JAIL_PATH_PREFIX);
        path.join(id.to_string()).join("root")
    }

    fn overlay_from_id(id: u32) -> String {
        format!("{}{}", OVERLAY_PREFIX, id)
    }

    fn find_loop_device_by_backing_file(backing_file: &Path) -> Result<Option<OsString>> {
        let sys_block_path = PathBuf::from("/sys/block");

        for entry in fs::read_dir(sys_block_path)? {
            let entry = entry?;
            let path = entry.path();

            if let Some(loop_name) = path
                .file_name()
                .and_then(|name| name.to_str())
                .filter(|&name| name.starts_with("loop"))
            {
                let backing_file_path = path.join("loop").join("backing_file");
                if let Ok(contents) = fs::read_to_string(&backing_file_path) {
                    if contents.trim() == backing_file.to_string_lossy() {
                        return Ok(Some(OsString::from(format!("/dev/{}", loop_name))));
                    }
                }
            }
        }

        Ok(None)
    }
}
