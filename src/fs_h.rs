use core::mem::size_of;

// On-disk file system format.
// Both the kernel and mkfs now use these definitions !

// Disk layout:
// [ boot block | super block | log | inode blocks | free bit map | data blocks ]
//
// mkfs computes the super block and builds an initial file system. The
// super block describes the disk layout:

pub const ROOTINO: u32 = 1; // root i-number
pub const BSIZE: usize = 512; // block size

pub const NDIRECT: usize = 12; // number of direct block pointers
pub const NINDIRECT: usize = BSIZE / size_of::<u32>(); // indirect pointers per block
pub const MAXFILE: usize = NDIRECT + NINDIRECT; // max file size in blocks

pub const DIRSIZ: usize = 14; // directory name length

pub const DINODE_SIZE: usize = size_of::<Dinode>();
pub const IPB: usize = BSIZE / DINODE_SIZE; // inodes per block
pub const BPB: usize = BSIZE * 8; // bitmap bits per block

pub const FSSIZE: u32 = 1000; // size of file system in blocks
pub const NINODES: u32 = 200; // number of inodes
pub const LOGSIZE: u32 = 0; // max data blocks in on-disk log

pub const T_DIR: u16 = 1; // directory
pub const T_FILE: u16 = 2; // file
#[allow(dead_code)]
pub const T_DEV: u16 = 3; // device

#[allow(dead_code)]
#[inline]
pub fn iblock(inum: u32, sb: &Superblock) -> u32 {
    inum / (IPB as u32) + sb.inodestart
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Superblock {
    pub size: u32,
    pub nblocks: u32,
    pub ninodes: u32,
    pub nlog: u32,
    pub logstart: u32,
    pub inodestart: u32,
    pub bmapstart: u32,
}

impl Superblock {
    pub const fn new() -> Self {
        Self {
            size: 0,
            nblocks: 0,
            ninodes: 0,
            nlog: 0,
            logstart: 0,
            inodestart: 0,
            bmapstart: 0,
        }
    }
}

impl Default for Superblock {
    fn default() -> Self {
        Self::new()
    }
}

// On-disk inode structure.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct Dinode {
    pub typ: u16,
    pub major: u16,
    pub minor: u16,
    pub nlink: u16,
    pub size: u32,
    pub addrs: [u32; NDIRECT + 1],
}

impl Dinode {
    pub const fn new() -> Self {
        Self {
            typ: 0,
            major: 0,
            minor: 0,
            nlink: 0,
            size: 0,
            addrs: [0; NDIRECT + 1],
        }
    }
}

impl Default for Dinode {
    fn default() -> Self {
        Self::new()
    }
}

// Directory is a file containing a sequence of dirent structures.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct Dirent {
    pub inum: u16,
    pub name: [u8; DIRSIZ],
}

impl Dirent {
    pub const fn new() -> Self {
        Self {
            inum: 0,
            name: [0; DIRSIZ],
        }
    }
}

impl Default for Dirent {
    fn default() -> Self {
        Self::new()
    }
}
