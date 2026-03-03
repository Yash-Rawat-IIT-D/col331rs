// On-disk file system format.
// Both the kernel and user programs use this header file (in C version).
// This is the Rust equivalent of fs.h

use crate::constants::*;

// Disk layout:
// [ boot block | super block | log | inode blocks |
//                                          free bit map | data blocks]
//
// mkfs computes the super block and builds an initial file system. The
// super block describes the disk layout:

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Superblock {
    pub size: u32,         // Size of file system image (blocks)
    pub nblocks: u32,      // Number of data blocks
    pub ninodes: u32,      // Number of inodes
    pub nlog: u32,         // Number of log blocks
    pub logstart: u32,     // Block number of first log block
    pub inodestart: u32,   // Block number of first inode block
    pub bmapstart: u32,    // Block number of first free map block
}

// On-disk inode structure
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Dinode {
    pub typ: u16,          // File type (use 'typ' as 'type' is a Rust keyword)
    pub major: u16,        // Major device number (T_DEV only)
    pub minor: u16,        // Minor device number (T_DEV only)
    pub nlink: u16,        // Number of links to inode in file system
    pub size: u32,         // Size of file (bytes)
    pub addrs: [u32; NDIRECT + 1],   // Data block addresses
}

// Directory entry structure
// A directory is a file containing a sequence of dirent structures.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Dirent {
    pub inum: u16,         // Inode number
    pub name: [u8; DIRSIZ], // Directory name (max DIRSIZ chars)
}

// Helper functions that match C macros from fs.h

/// Inodes per block: IPB = (BSIZE / sizeof(struct dinode))
pub const fn inodes_per_block() -> usize {
    BSIZE / core::mem::size_of::<Dinode>()
}

/// Block containing inode i: IBLOCK(i, sb) = ((i) / IPB + sb.inodestart)
pub fn iblock(inum: u32, sb: &Superblock) -> u32 {
    (inum / inodes_per_block() as u32) + u32::from_le(sb.inodestart)
}

/// Block of free map containing bit for block b: BBLOCK(b, sb) = (b/BPB + sb.bmapstart)
pub fn bblock(blockno: u32, sb: &Superblock) -> u32 {
    (blockno / BPB as u32) + u32::from_le(sb.bmapstart)
}

// Default implementations for convenience
impl Default for Superblock {
    fn default() -> Self {
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

impl Default for Dinode {
    fn default() -> Self {
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

impl Default for Dirent {
    fn default() -> Self {
        Self {
            inum: 0,
            name: [0; DIRSIZ],
        }
    }
}
