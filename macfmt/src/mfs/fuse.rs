use std::ffi::OsStr;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use libc::{EINVAL, ENOENT};
use fuser::{Filesystem, ReplyCreate, ReplyData, Request, MountOption, FileType, TimeOrNow, ReplyEntry, ReplyAttr, ReplyDirectory, FileAttr};

use crate::mfs::{FileDirectoryBlock, Mfs};
use crate::common::{DateTime, PascalString, BootBlocks, SizedString, DynamicPascalString};

#[derive(Clone, Debug)]
pub struct MfsFuse(Mfs);

impl MfsFuse {
    pub fn new(fs: Mfs) -> MfsFuse {
        MfsFuse(fs)
    }
    pub fn mount(self, dir: &Path) -> std::io::Result<()> {
        let opts = [
            MountOption::FSName("mfs".to_string()),
        ];
        fuser::mount2(self, dir, &opts)
    }
}

const TTL: std::time::Duration = std::time::Duration::from_secs(1);

const ROOT_DIR_ATTR: FileAttr = FileAttr {
    ino: 1,
    size: 0,
    blocks: 0,
    atime: UNIX_EPOCH, // 1970-01-01 00:00:00
    mtime: UNIX_EPOCH,
    ctime: UNIX_EPOCH,
    crtime: UNIX_EPOCH,
    kind: FileType::Directory,
    perm: 0o755,
    nlink: 2,
    uid: 501,
    gid: 20,
    rdev: 0,
    flags: 0,
    blksize: 512,
};

const FILE_ATTR: FileAttr = FileAttr {
    ino: 0,
    size: 0,
    blocks: 0,
    atime: UNIX_EPOCH, // 1970-01-01 00:00:00
    mtime: UNIX_EPOCH,
    ctime: UNIX_EPOCH,
    crtime: UNIX_EPOCH,
    kind: FileType::RegularFile,
    perm: 0o755,
    nlink: 2,
    uid: 501,
    gid: 20,
    rdev: 0,
    flags: 0,
    blksize: 512,
};

impl MfsFuse {
    fn root_attr(&self) -> FileAttr {
        let mut attr = ROOT_DIR_ATTR.clone();

        attr.atime = self.0.creation_date();
        attr.mtime = self.0.creation_date();
        attr.ctime = self.0.creation_date();
        attr.crtime = self.0.creation_date();

        attr
    }
    fn file_attr(&self, file: &FileDirectoryBlock, is_rsrc: bool) -> FileAttr {
        let div_ceil = |x: u32, y: u32| (x+y-1)/y;
        let mut attr = FILE_ATTR.clone();
        attr.atime = file.modification_date();
        attr.mtime = file.modification_date();
        attr.ctime = file.creation_date();
        attr.crtime = file.creation_date();
        attr.blocks = div_ceil(file.data_fork_size(), self.0.alloc_block_size()) as u64;
        attr.size = file.data_fork_size() as u64;
        attr.ino = self.ino_by_file(file, is_rsrc) as u64;

        attr
    }

    fn ino_by_file(&self, file: &FileDirectoryBlock, is_rsrc: bool) -> i64 {
        let ino = file.number() << 1 | (is_rsrc as u8) as u32;

        ino as i64 + 1
    }

    fn file_by_ino(&self, mut ino: i64) -> Option<(&FileDirectoryBlock, bool)> {
        if ino == 1 {
            return None;
        }
        ino -= 1;

        self.0.file_by_id((ino >> 1) as u32).map(|v| (v, ino & 1 == 1))
    }
}

impl Filesystem for MfsFuse {
    fn lookup(&mut self, _req: &Request, parent: u64, name: &OsStr, reply: ReplyEntry) {
        if parent != 1 {
            reply.error(ENOENT);
            return;
        }
        let name_str = name.to_string_lossy();
        let (name, is_rsrc) = if let Some(split) = name_str.strip_suffix(".rsrc") {
            (split.to_string(), true)
        } else {
            (name_str.to_string(), false)
        };

        let Some(file) = self.0.file_by_name(&name) else {
            reply.error(ENOENT);
            return;
        };
        
        reply.entry(&TTL, &self.file_attr(file, is_rsrc), 0);
    }
    fn getattr(&mut self, _req: &Request, ino: u64, _fh: Option<u64>, reply: ReplyAttr) {
        match ino {
            1 => reply.attr(&TTL, &self.root_attr()),
            _ => {
                let Some((file, is_rsrc)) = self.file_by_ino(ino as i64) else {
                    reply.error(ENOENT);
                    return;
                };

                reply.attr(&TTL, &self.file_attr(file, is_rsrc));
            },
        }
    }
    fn create(
        &mut self,
        _req: &Request,
        parent: u64,
        name: &OsStr,
        _mode: u32,
        _umask: u32,
        _flags: i32,
        reply: ReplyCreate,
    ) {
        let name = name.to_string_lossy().to_owned();
        self.0.add_file(&name.clone(), *b"DUPA", *b"MAJA");
        let file = self.0.file_by_name(&name).unwrap();
        let attr = self.file_attr(&file, false);
        reply.created(&TTL, &attr, 0, 0, 0);
    }
    fn setattr(
        &mut self,
        _req: &Request<'_>,
        ino: u64,
        mode: Option<u32>,
        uid: Option<u32>,
        gid: Option<u32>,
        size: Option<u64>,
        _atime: Option<TimeOrNow>,
        _mtime: Option<TimeOrNow>,
        _ctime: Option<SystemTime>,
        fh: Option<u64>,
        _crtime: Option<SystemTime>,
        _chgtime: Option<SystemTime>,
        _bkuptime: Option<SystemTime>,
        flags: Option<u32>,
        reply: ReplyAttr,
    ) {
        match ino {
            1 => reply.attr(&TTL, &self.root_attr()),
            _ => {
                let Some((file, is_rsrc)) = self.file_by_ino(ino as i64) else {
                    reply.error(ENOENT);
                    return;
                };

                reply.attr(&TTL, &self.file_attr(file, is_rsrc));
            },
        }
    }
    fn read(
        &mut self,
        _req: &Request,
        ino: u64,
        _fh: u64,
        offset: i64,
        size: u32,
        _flags: i32,
        _lock: Option<u64>,
        reply: ReplyData,
    ) {
        match ino {
            1 => reply.error(EINVAL),
            _ => {
                let Some((file, use_rsrc)) = self.file_by_ino(ino as i64) else {
                    reply.error(ENOENT);
                    return;
                };
                let data = if use_rsrc {
                    self.0.file_rsrc(file)
                } else {
                    self.0.file_data(file)
                };
                let len = if offset + size as i64 > data.len() as i64 { data.len() - offset as usize} else { size as usize };
                reply.data(&data[offset as usize..][..len]);
            },
        }
    }

    fn readdir(
        &mut self,
        _req: &Request,
        ino: u64,
        _fh: u64,
        offset: i64,
        mut reply: ReplyDirectory,
    ) {
        if ino != 1 {
            reply.error(ENOENT);
            return;
        }

        let mut entries = vec![
            (1, FileType::Directory, ".".to_string()),
            (1, FileType::Directory, "..".to_string()),
        ];

        for f in self.0.files() {
            if f.resource_fork_size() != 0 {
                entries.push((
                    self.ino_by_file(f, true),
                    FileType::RegularFile,
                    format!("{}.rsrc", f.name().to_string()),
                ));
            }

            if f.data_fork_size() != 0 || f.resource_fork_size() == 0 {
                entries.push((self.ino_by_file(f, false), FileType::RegularFile, f.name().to_string()));
            }
        }

        for (i, entry) in entries.into_iter().enumerate().skip(offset as usize) {
            // i + 1 means the index of the next entry
            if reply.add(entry.0 as u64, (i + 1) as i64, entry.1, entry.2) {
                break;
            }
        }
        reply.ok();
    }
}

