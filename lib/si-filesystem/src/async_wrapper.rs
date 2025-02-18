use std::path::Path;

use fuser::{
    Filesystem, ReplyAttr, ReplyBmap, ReplyCreate, ReplyData, ReplyDirectory, ReplyEmpty,
    ReplyEntry, ReplyLock, ReplyWrite, ReplyXattr,
};
use tokio::sync::mpsc::UnboundedSender;

use crate::{FileHandle, FilesystemCommand, Inode};

pub struct AsyncFuseWrapper {
    tx: UnboundedSender<FilesystemCommand>,
}

impl AsyncFuseWrapper {
    pub fn new(tx: UnboundedSender<FilesystemCommand>) -> Self {
        Self { tx }
    }
}

impl Filesystem for AsyncFuseWrapper {
    fn getattr(&mut self, _req: &fuser::Request<'_>, ino: u64, fh: Option<u64>, reply: ReplyAttr) {
        self.tx
            .send(FilesystemCommand::GetAttr {
                ino: Inode::new(ino),
                fh: fh.map(FileHandle::new),
                reply,
            })
            .unwrap();
    }

    fn readdir(
        &mut self,
        _req: &fuser::Request<'_>,
        ino: u64,
        fh: u64,
        offset: i64,
        reply: ReplyDirectory,
    ) {
        self.tx
            .send(FilesystemCommand::ReadDir {
                ino: Inode::new(ino),
                fh: FileHandle::new(fh),
                offset,
                reply,
            })
            .unwrap();
    }

    fn init(
        &mut self,
        _req: &fuser::Request<'_>,
        _config: &mut fuser::KernelConfig,
    ) -> Result<(), nix::libc::c_int> {
        self.tx.send(FilesystemCommand::HydrateChangeSets).unwrap();
        Ok(())
    }

    fn destroy(&mut self) {}

    fn lookup(
        &mut self,
        _req: &fuser::Request<'_>,
        parent: u64,
        name: &std::ffi::OsStr,
        reply: ReplyEntry,
    ) {
        self.tx
            .send(FilesystemCommand::Lookup {
                parent: Inode::new(parent),
                name: name.to_os_string(),
                reply,
            })
            .unwrap();
    }

    fn forget(&mut self, _req: &fuser::Request<'_>, ino: u64, nlookup: u64) {
        self.tx
            .send(FilesystemCommand::Forget {
                ino: Inode::new(ino),
                nlookup,
            })
            .unwrap();
    }

    fn setattr(
        &mut self,
        _req: &fuser::Request<'_>,
        ino: u64,
        mode: Option<u32>,
        uid: Option<u32>,
        gid: Option<u32>,
        size: Option<u64>,
        _atime: Option<fuser::TimeOrNow>,
        _mtime: Option<fuser::TimeOrNow>,
        _ctime: Option<std::time::SystemTime>,
        fh: Option<u64>,
        _crtime: Option<std::time::SystemTime>,
        _chgtime: Option<std::time::SystemTime>,
        _bkuptime: Option<std::time::SystemTime>,
        flags: Option<u32>,
        reply: ReplyAttr,
    ) {
        self.tx
            .send(FilesystemCommand::SetAttr {
                ino: Inode::new(ino),
                mode,
                uid,
                gid,
                size,
                fh: fh.map(FileHandle::new),
                flags,
                reply,
            })
            .unwrap();
    }

    fn readlink(&mut self, _req: &fuser::Request<'_>, ino: u64, reply: ReplyData) {
        self.tx
            .send(FilesystemCommand::ReadLink {
                ino: Inode::new(ino),
                reply,
            })
            .unwrap();
    }

    fn mknod(
        &mut self,
        _req: &fuser::Request<'_>,
        parent: u64,
        name: &std::ffi::OsStr,
        mode: u32,
        umask: u32,
        rdev: u32,
        reply: ReplyEntry,
    ) {
        self.tx
            .send(FilesystemCommand::MkNod {
                parent: Inode::new(parent),
                name: name.to_os_string(),
                mode,
                umask,
                rdev,
                reply,
            })
            .unwrap();
    }

    fn mkdir(
        &mut self,
        _req: &fuser::Request<'_>,
        parent: u64,
        name: &std::ffi::OsStr,
        mode: u32,
        umask: u32,
        reply: ReplyEntry,
    ) {
        self.tx
            .send(FilesystemCommand::MkDir {
                parent: Inode::new(parent),
                name: name.to_os_string(),
                mode,
                umask,
                reply,
            })
            .unwrap();
    }

    fn unlink(
        &mut self,
        _req: &fuser::Request<'_>,
        parent: u64,
        name: &std::ffi::OsStr,
        reply: ReplyEmpty,
    ) {
        self.tx
            .send(FilesystemCommand::Unlink {
                parent: Inode::new(parent),
                name: name.to_os_string(),
                reply,
            })
            .unwrap();
    }

    fn rmdir(
        &mut self,
        _req: &fuser::Request<'_>,
        parent: u64,
        name: &std::ffi::OsStr,
        reply: ReplyEmpty,
    ) {
        self.tx
            .send(FilesystemCommand::RmDir {
                parent: Inode::new(parent),
                name: name.to_os_string(),
                reply,
            })
            .unwrap();
    }

    fn symlink(
        &mut self,
        _req: &fuser::Request<'_>,
        parent: u64,
        link_name: &std::ffi::OsStr,
        target: &Path,
        reply: ReplyEntry,
    ) {
        self.tx
            .send(FilesystemCommand::SymLink {
                parent: Inode::new(parent),
                link_name: link_name.to_os_string(),
                target: target.to_path_buf(),
                reply,
            })
            .unwrap();
    }

    fn rename(
        &mut self,
        _req: &fuser::Request<'_>,
        parent: u64,
        name: &std::ffi::OsStr,
        newparent: u64,
        newname: &std::ffi::OsStr,
        flags: u32,
        reply: ReplyEmpty,
    ) {
        self.tx
            .send(FilesystemCommand::Rename {
                parent: Inode::new(parent),
                name: name.to_os_string(),
                newparent: Inode::new(newparent),
                newname: newname.to_os_string(),
                flags,
                reply,
            })
            .unwrap();
    }

    fn link(
        &mut self,
        _req: &fuser::Request<'_>,
        ino: u64,
        newparent: u64,
        newname: &std::ffi::OsStr,
        reply: ReplyEntry,
    ) {
        self.tx
            .send(FilesystemCommand::Link {
                ino: Inode::new(ino),
                newparent: Inode::new(newparent),
                newname: newname.to_os_string(),
                reply,
            })
            .unwrap();
    }

    fn open(&mut self, _req: &fuser::Request<'_>, ino: u64, flags: i32, reply: fuser::ReplyOpen) {
        self.tx
            .send(FilesystemCommand::Open {
                ino: Inode::new(ino),
                flags,
                reply,
            })
            .unwrap();
        // reply.opened(0, 0);
    }

    fn read(
        &mut self,
        _req: &fuser::Request<'_>,
        ino: u64,
        fh: u64,
        offset: i64,
        size: u32,
        flags: i32,
        lock_owner: Option<u64>,
        reply: ReplyData,
    ) {
        self.tx
            .send(FilesystemCommand::Read {
                ino: Inode::new(ino),
                fh: FileHandle::new(fh),
                offset,
                size,
                flags,
                lock_owner,
                reply,
            })
            .unwrap();
    }

    fn write(
        &mut self,
        _req: &fuser::Request<'_>,
        ino: u64,
        fh: u64,
        offset: i64,
        data: &[u8],
        write_flags: u32,
        flags: i32,
        lock_owner: Option<u64>,
        reply: ReplyWrite,
    ) {
        self.tx
            .send(FilesystemCommand::Write {
                ino: Inode::new(ino),
                fh: FileHandle::new(fh),
                offset,
                data: data.to_vec(),
                write_flags,
                flags,
                lock_owner,
                reply,
            })
            .unwrap();
    }

    fn flush(
        &mut self,
        _req: &fuser::Request<'_>,
        ino: u64,
        fh: u64,
        lock_owner: u64,
        reply: ReplyEmpty,
    ) {
        self.tx
            .send(FilesystemCommand::Flush {
                ino: Inode::new(ino),
                fh: FileHandle::new(fh),
                lock_owner,
                reply,
            })
            .unwrap();
    }

    fn release(
        &mut self,
        _req: &fuser::Request<'_>,
        ino: u64,
        fh: u64,
        flags: i32,
        lock_owner: Option<u64>,
        flush: bool,
        reply: ReplyEmpty,
    ) {
        self.tx
            .send(FilesystemCommand::Release {
                ino: Inode::new(ino),
                fh: FileHandle::new(fh),
                flags,
                lock_owner,
                flush,
                reply,
            })
            .unwrap();
    }

    fn fsync(
        &mut self,
        _req: &fuser::Request<'_>,
        ino: u64,
        fh: u64,
        datasync: bool,
        reply: ReplyEmpty,
    ) {
        self.tx
            .send(FilesystemCommand::FSync {
                ino: Inode::new(ino),
                fh: FileHandle::new(fh),
                datasync,
                reply,
            })
            .unwrap();
    }

    fn opendir(
        &mut self,
        _req: &fuser::Request<'_>,
        ino: u64,
        flags: i32,
        reply: fuser::ReplyOpen,
    ) {
        self.tx
            .send(FilesystemCommand::OpenDir {
                ino: Inode::new(ino),
                flags,
                reply,
            })
            .unwrap();
    }

    fn readdirplus(
        &mut self,
        _req: &fuser::Request<'_>,
        ino: u64,
        fh: u64,
        offset: i64,
        reply: fuser::ReplyDirectoryPlus,
    ) {
        self.tx
            .send(FilesystemCommand::ReadDirPlus {
                ino: Inode::new(ino),
                fh: FileHandle::new(fh),
                offset,
                reply,
            })
            .unwrap();
    }

    fn releasedir(
        &mut self,
        _req: &fuser::Request<'_>,
        ino: u64,
        fh: u64,
        flags: i32,
        reply: ReplyEmpty,
    ) {
        self.tx
            .send(FilesystemCommand::ReleaseDir {
                ino: Inode::new(ino),
                fh: FileHandle::new(fh),
                flags,
                reply,
            })
            .unwrap();
        // reply.ok();
    }

    fn fsyncdir(
        &mut self,
        _req: &fuser::Request<'_>,
        ino: u64,
        fh: u64,
        datasync: bool,
        reply: ReplyEmpty,
    ) {
        self.tx
            .send(FilesystemCommand::FSyncDir {
                ino: Inode::new(ino),
                fh: FileHandle::new(fh),
                datasync,
                reply,
            })
            .unwrap();
    }

    fn statfs(&mut self, _req: &fuser::Request<'_>, _ino: u64, reply: fuser::ReplyStatfs) {
        reply.statfs(0, 0, 0, 0, 0, 512, 255, 0);
    }

    fn setxattr(
        &mut self,
        _req: &fuser::Request<'_>,
        ino: u64,
        name: &std::ffi::OsStr,
        value: &[u8],
        flags: i32,
        position: u32,
        reply: ReplyEmpty,
    ) {
        self.tx
            .send(FilesystemCommand::SetXattr {
                ino: Inode::new(ino),
                name: name.to_os_string(),
                value: value.to_vec(),
                flags,
                position,
                reply,
            })
            .unwrap();
    }

    fn getxattr(
        &mut self,
        _req: &fuser::Request<'_>,
        ino: u64,
        name: &std::ffi::OsStr,
        size: u32,
        reply: ReplyXattr,
    ) {
        self.tx
            .send(FilesystemCommand::GetXattr {
                ino: Inode::new(ino),
                name: name.to_os_string(),
                size,
                reply,
            })
            .unwrap();
    }

    fn listxattr(&mut self, _req: &fuser::Request<'_>, ino: u64, size: u32, reply: ReplyXattr) {
        self.tx
            .send(FilesystemCommand::ListXattr {
                ino: Inode::new(ino),
                size,
                reply,
            })
            .unwrap();
    }

    fn removexattr(
        &mut self,
        _req: &fuser::Request<'_>,
        ino: u64,
        name: &std::ffi::OsStr,
        reply: ReplyEmpty,
    ) {
        self.tx
            .send(FilesystemCommand::RemoveXattr {
                ino: Inode::new(ino),
                name: name.to_os_string(),
                reply,
            })
            .unwrap();
    }

    fn access(&mut self, _req: &fuser::Request<'_>, ino: u64, mask: i32, reply: ReplyEmpty) {
        self.tx
            .send(FilesystemCommand::Access {
                ino: Inode::new(ino),
                mask,
                reply,
            })
            .unwrap();
    }

    fn create(
        &mut self,
        _req: &fuser::Request<'_>,
        parent: u64,
        name: &std::ffi::OsStr,
        mode: u32,
        umask: u32,
        flags: i32,
        reply: ReplyCreate,
    ) {
        self.tx
            .send(FilesystemCommand::Create {
                parent: Inode::new(parent),
                name: name.to_os_string(),
                mode,
                umask,
                flags,
                reply,
            })
            .unwrap();
    }

    fn getlk(
        &mut self,
        _req: &fuser::Request<'_>,
        ino: u64,
        fh: u64,
        lock_owner: u64,
        start: u64,
        end: u64,
        typ: i32,
        pid: u32,
        reply: ReplyLock,
    ) {
        self.tx
            .send(FilesystemCommand::GetLk {
                ino: Inode::new(ino),
                fh: FileHandle::new(fh),
                lock_owner,
                start,
                end,
                typ,
                pid,
                reply,
            })
            .unwrap();
    }

    fn setlk(
        &mut self,
        _req: &fuser::Request<'_>,
        ino: u64,
        fh: u64,
        lock_owner: u64,
        start: u64,
        end: u64,
        typ: i32,
        pid: u32,
        sleep: bool,
        reply: ReplyEmpty,
    ) {
        self.tx
            .send(FilesystemCommand::SetLk {
                ino: Inode::new(ino),
                fh: FileHandle::new(fh),
                lock_owner,
                start,
                end,
                typ,
                pid,
                sleep,
                reply,
            })
            .unwrap();
    }

    fn bmap(
        &mut self,
        _req: &fuser::Request<'_>,
        ino: u64,
        blocksize: u32,
        idx: u64,
        reply: ReplyBmap,
    ) {
        self.tx
            .send(FilesystemCommand::Bmap {
                ino: Inode::new(ino),
                blocksize,
                idx,
                reply,
            })
            .unwrap()
    }

    fn ioctl(
        &mut self,
        _req: &fuser::Request<'_>,
        ino: u64,
        fh: u64,
        flags: u32,
        cmd: u32,
        in_data: &[u8],
        out_size: u32,
        reply: fuser::ReplyIoctl,
    ) {
        self.tx
            .send(FilesystemCommand::IoCtl {
                ino: Inode::new(ino),
                fh: FileHandle::new(fh),
                flags,
                cmd,
                in_data: in_data.to_vec(),
                out_size,
                reply,
            })
            .unwrap();
    }

    fn fallocate(
        &mut self,
        _req: &fuser::Request<'_>,
        ino: u64,
        fh: u64,
        offset: i64,
        length: i64,
        mode: i32,
        reply: ReplyEmpty,
    ) {
        self.tx
            .send(FilesystemCommand::Fallocate {
                ino: Inode::new(ino),
                fh: FileHandle::new(fh),
                offset,
                length,
                mode,
                reply,
            })
            .unwrap();
    }

    fn lseek(
        &mut self,
        _req: &fuser::Request<'_>,
        ino: u64,
        fh: u64,
        offset: i64,
        whence: i32,
        reply: fuser::ReplyLseek,
    ) {
        self.tx
            .send(FilesystemCommand::Lseek {
                ino: Inode::new(ino),
                fh: FileHandle::new(fh),
                offset,
                whence,
                reply,
            })
            .unwrap();
    }

    fn copy_file_range(
        &mut self,
        _req: &fuser::Request<'_>,
        ino_in: u64,
        fh_in: u64,
        offset_in: i64,
        ino_out: u64,
        fh_out: u64,
        offset_out: i64,
        len: u64,
        flags: u32,
        reply: ReplyWrite,
    ) {
        self.tx
            .send(FilesystemCommand::CopyFileRange {
                ino_in: Inode::new(ino_in),
                fh_in: FileHandle::new(fh_in),
                offset_in,
                ino_out: Inode::new(ino_out),
                fh_out: FileHandle::new(fh_out),
                offset_out,
                len,
                flags,
                reply,
            })
            .unwrap();
    }
}
