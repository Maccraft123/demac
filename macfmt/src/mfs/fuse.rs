use std::ffi::OsStr;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use libc::{EINVAL, ENOENT};
use fuser::{Filesystem, ReplyCreate, ReplyData, Request, MountOption, FileType, TimeOrNow, ReplyEntry, ReplyAttr, ReplyDirectory, FileAttr, ReplyWrite};

use crate::mfs::{FileDirectoryBlock, Mfs, Fork};
use crate::common::{DateTime, PascalString, BootBlocks, SizedString, DynamicPascalString};

#[derive(Clone, Debug)]
pub struct MfsFuse {
    fs: Mfs,
}

impl MfsFuse {
    pub fn new(fs: Mfs) -> MfsFuse {
        MfsFuse {
            fs
        }
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

        attr.atime = self.fs.creation_date();
        attr.mtime = self.fs.creation_date();
        attr.ctime = self.fs.creation_date();
        attr.crtime = self.fs.creation_date();

        attr
    }
    fn file_attr(&self, file: &FileDirectoryBlock, fork: Fork) -> FileAttr {
        let div_ceil = |x: u32, y: u32| (x+y-1)/y;
        let mut attr = FILE_ATTR.clone();
        attr.atime = file.modification_date();
        attr.mtime = file.modification_date();
        attr.ctime = file.creation_date();
        attr.crtime = file.creation_date();
        match fork {
            Fork::Data => {
                attr.blocks = div_ceil(file.data_fork_size(), self.fs.alloc_block_size()) as u64;
                attr.size = file.data_fork_size() as u64;
            },
            Fork::Resource => {
                attr.blocks = div_ceil(file.resource_fork_size(), self.fs.alloc_block_size()) as u64;
                attr.size = file.resource_fork_size() as u64;
            },
        }
        attr.ino = self.ino_by_file(file, fork) as u64;

        attr
    }

    fn ino_by_file(&self, file: &FileDirectoryBlock, fork: Fork) -> u64 {
        ((file.number() as u64) << 1 | ((fork == Fork::Data) as u8) as u64 ) + 1
    }

    fn file_by_ino(&self, mut ino: i64) -> Option<(&FileDirectoryBlock, Fork)> {
        if ino == 1 {
            return None;
        }
        ino -= 1;
        let fork = if ino & 1 == 1 {
            Fork::Data
        } else {
            Fork::Resource
        };

        self.fs.file_by_id((ino >> 1) as u32)
            .map(|v| (v, fork))
    }

    fn name_to_fork<'a>(&self, name: &'a impl AsRef<str>) -> (&'a str, Fork) {
        if let Some(filename) = name.as_ref().strip_suffix(".rsrc") {
            (filename, Fork::Resource)
        } else {
            (name.as_ref(), Fork::Data)
        }
    }
}

impl Filesystem for MfsFuse {
    fn lookup(&mut self, _req: &Request, parent: u64, name: &OsStr, reply: ReplyEntry) {
        if parent != 1 {
            reply.error(ENOENT);
            return;
        }
        let name_str = name.to_string_lossy();
        let (name, fork) = self.name_to_fork(&name_str);

        let Some(file) = self.fs.file_by_name(&name) else {
            reply.error(ENOENT);
            return;
        };
        
        reply.entry(&TTL, &self.file_attr(file, fork), 0);
    }
    fn getattr(&mut self, _req: &Request, ino: u64, _fh: Option<u64>, reply: ReplyAttr) {
        match ino {
            1 => reply.attr(&TTL, &self.root_attr()),
            _ => {
                let Some((file, fork)) = self.file_by_ino(ino as i64) else {
                    reply.error(ENOENT);
                    return;
                };

                reply.attr(&TTL, &self.file_attr(file, fork));
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
        let name_str = name.to_string_lossy();
        let (name, fork) = self.name_to_fork(&name_str);

        self.fs.add_file(&name.clone(), *b"DUPA", *b"MAJA");
        let file = self.fs.file_by_name(&name).unwrap();
        let attr = self.file_attr(&file, fork);
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
                let Some((file, fork)) = self.file_by_ino(ino as i64) else {
                    reply.error(ENOENT);
                    return;
                };

                reply.attr(&TTL, &self.file_attr(file, fork));
            },
        }
    }
    fn write(
        &mut self,
        _req: &Request,
        ino: u64,
        fh: u64,
        offset: i64,
        data: &[u8],
        _write_flags: u32,
        #[allow(unused_variables)] flags: i32,
        _lock_owner: Option<u64>,
        reply: ReplyWrite
    ) {
        match ino {
            1 => reply.error(EINVAL),
            _ => {
                let Some((file, fork)) = self.file_by_ino(ino as i64) else {
                    reply.error(ENOENT);
                    return;
                };
                let file = file.clone();
                let mut vec: Vec<u8> = data.to_vec();
                self.fs.append_file_data(&file, &mut vec);
                reply.written(data.len() as u32);
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
                let Some((file, fork)) = self.file_by_ino(ino as i64) else {
                    reply.error(ENOENT);
                    return;
                };
                let data = match fork {
                    Fork::Resource => self.fs.file_rsrc(file),
                    Fork::Data => self.fs.file_data(file),
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

        for f in self.fs.files() {
            if f.resource_fork_size() != 0 {
                entries.push((
                    self.ino_by_file(f, Fork::Resource),
                    FileType::RegularFile,
                    format!("{}.rsrc", f.name().to_string()),
                ));
            }

            if f.data_fork_size() != 0 || f.resource_fork_size() == 0 {
                entries.push((self.ino_by_file(f, Fork::Data), FileType::RegularFile, f.name().to_string()));
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

