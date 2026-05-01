use core::cmp::min;

use crate::bio;
use crate::buf::BSIZE;
use crate::fs_h::{self, DINODE_SIZE, DIRSIZ, IPB, NDIRECT, NINDIRECT, Superblock, T_DIR};
use crate::param::NINODE;
use crate::println;

pub use crate::fs_h::{Dirent, ROOTINO};
pub const DIRENT_SIZE: usize = core::mem::size_of::<Dirent>();

#[repr(C)]
pub struct Inode {
    pub dev: u32,
    pub inum: u32,
    pub refcnt: i32,
    pub valid: i32,

    pub type_: i16,
    pub major: i16,
    pub minor: i16,
    pub nlink: i16,
    pub size: u32,
    pub addrs: [u32; NDIRECT + 1],
}

impl Inode {
    pub const fn new() -> Self {
        Self {
            dev: 0,
            inum: 0,
            refcnt: 0,
            valid: 0,
            type_: 0,
            major: 0,
            minor: 0,
            nlink: 0,
            size: 0,
            addrs: [0; NDIRECT + 1],
        }
    }
}

#[repr(C)]
pub struct Stat {
    pub type_: i16,
    pub dev: i32,
    pub ino: u32,
    pub nlink: i16,
    pub size: u32,
}

impl Stat {
    pub const fn new() -> Self {
        Self {
            type_: 0,
            dev: 0,
            ino: 0,
            nlink: 0,
            size: 0,
        }
    }
}

struct ICache {
    inode: [Inode; NINODE],
}

impl ICache {
    const fn new() -> Self {
        Self {
            inode: [const { Inode::new() }; NINODE],
        }
    }
}

static mut SB: Superblock = Superblock::new();
static mut ICACHE: ICache = ICache::new();

#[inline]
fn read_u16_le(data: &[u8], off: usize) -> u16 {
    u16::from_le_bytes([data[off], data[off + 1]])
}

#[inline]
fn read_i16_le(data: &[u8], off: usize) -> i16 {
    i16::from_le_bytes([data[off], data[off + 1]])
}

#[inline]
fn read_u32_le(data: &[u8], off: usize) -> u32 {
    u32::from_le_bytes([data[off], data[off + 1], data[off + 2], data[off + 3]])
}

#[inline]
fn iblock(inum: u32, sb: &Superblock) -> u32 {
    fs_h::iblock(inum, sb)
}

pub fn parse_dirent(raw: &[u8]) -> Dirent {
    let mut de = Dirent::new();
    de.inum = read_u16_le(raw, 0);
    de.name.copy_from_slice(&raw[2..2 + DIRSIZ]);
    de
}

pub fn readsb(dev: u32, sb: &mut Superblock) {
    let bp = bio::bread(dev, 1);
    let data = &bio::buf_mut(bp).data;

    sb.size = read_u32_le(data, 0);
    sb.nblocks = read_u32_le(data, 4);
    sb.ninodes = read_u32_le(data, 8);
    sb.nlog = read_u32_le(data, 12);
    sb.logstart = read_u32_le(data, 16);
    sb.inodestart = read_u32_le(data, 20);
    sb.bmapstart = read_u32_le(data, 24);

    bio::brelse(bp);
}

pub fn iinit(dev: u32) {
    unsafe {
        readsb(dev, &mut *(&raw mut SB));
        let sb = &*(&raw const SB);
        println!(
            "sb: size {} nblocks {} ninodes {} nlog {} logstart {} inodestart {} bmap start {}",
            sb.size, sb.nblocks, sb.ninodes, sb.nlog, sb.logstart, sb.inodestart, sb.bmapstart
        );
    }
}

/// Decrement reference count of inode at index idx.
/// Mirrors C: static void irelse(struct inode *ip) { ip->ref--; }
fn irelse(idx: usize) {
    unsafe {
        if idx >= NINODE {
            panic!("irelse: bad inode index");
        }
        ICACHE.inode[idx].refcnt -= 1;
    }
}

pub fn iget(dev: u32, inum: u32) -> usize {
    unsafe {
        let mut empty: Option<usize> = None;

        for i in 0..NINODE {
            let ip = &mut ICACHE.inode[i];

            if ip.refcnt > 0 && ip.dev == dev && ip.inum == inum {
                ip.refcnt += 1;
                return i;
            }

            if empty.is_none() && ip.refcnt == 0 {
                empty = Some(i);
            }
        }

        let idx = empty.unwrap_or_else(|| panic!("iget: no inodes"));
        let ip = &mut ICACHE.inode[idx];

        ip.dev = dev;
        ip.inum = inum;
        ip.refcnt = 1;
        ip.valid = 0;

        idx
    }
}

pub fn iread(idx: usize) {
    unsafe {
        if idx >= NINODE {
            panic!("iread: bad inode index");
        }

        let ip = &mut ICACHE.inode[idx];
        if ip.refcnt < 1 {
            panic!("iread");
        }

        if ip.valid == 0 {
            let bp = bio::bread(ip.dev, iblock(ip.inum, &*(&raw const SB)));
            let data = &bio::buf_mut(bp).data;

            let off = (ip.inum % (IPB as u32)) as usize * DINODE_SIZE;
            ip.type_ = read_i16_le(data, off);
            ip.major = read_i16_le(data, off + 2);
            ip.minor = read_i16_le(data, off + 4);
            ip.nlink = read_i16_le(data, off + 6);
            ip.size = read_u32_le(data, off + 8);

            for i in 0..(NDIRECT + 1) {
                ip.addrs[i] = read_u32_le(data, off + 12 + (i * 4));
            }

            bio::brelse(bp);

            ip.valid = 1;
            if ip.type_ == 0 {
                panic!("iread: no type");
            }
        }
    }
}

fn bmap(ip: &Inode, bn: u32) -> u32 {
    if (bn as usize) < NDIRECT {
        return ip.addrs[bn as usize];
    }

    let bn = bn - (NDIRECT as u32);
    if (bn as usize) < NINDIRECT {
        let bp = bio::bread(ip.dev, ip.addrs[NDIRECT]);
        let addr = {
            let data = &bio::buf_mut(bp).data;
            read_u32_le(data, (bn as usize) * 4)
        };
        bio::brelse(bp);
        return addr;
    }

    panic!("bmap: out of range");
}

pub fn stati(idx: usize, st: &mut Stat) {
    unsafe {
        if idx >= NINODE {
            panic!("stati: bad inode index");
        }

        let ip = &ICACHE.inode[idx];
        st.dev = ip.dev as i32;
        st.ino = ip.inum;
        st.type_ = ip.type_;
        st.nlink = ip.nlink;
        st.size = ip.size;
    }
}

pub fn readi(idx: usize, dst: &mut [u8], off: u32, n: u32) -> i32 {
    unsafe {
        if idx >= NINODE {
            panic!("readi: bad inode index");
        }

        let ip = &ICACHE.inode[idx];
        if off > ip.size || off.checked_add(n).is_none() {
            return -1;
        }

        let mut n = n;
        if off + n > ip.size {
            n = ip.size - off;
        }

        if (n as usize) > dst.len() {
            panic!("readi: destination too small");
        }

        let mut tot: u32 = 0;
        let mut cur_off = off;

        while tot < n {
            let bp = bio::bread(ip.dev, bmap(ip, cur_off / (BSIZE as u32)));

            let m = min((n - tot) as usize, BSIZE - (cur_off as usize % BSIZE));
            let boff = cur_off as usize % BSIZE;
            dst[tot as usize..tot as usize + m]
                .copy_from_slice(&bio::buf_mut(bp).data[boff..boff + m]);
            bio::brelse(bp);

            tot += m as u32;
            cur_off += m as u32;
        }

        n as i32
    }
}

// Directories
/// Compare names up to DIRSIZ.
pub fn namecmp(s: &[u8], t: &[u8]) -> bool {
    let slen = s.iter().take(DIRSIZ).position(|&b| b == 0).unwrap_or(DIRSIZ.min(s.len()));
    let tlen = t.iter().take(DIRSIZ).position(|&b| b == 0).unwrap_or(DIRSIZ.min(t.len()));
    if slen != tlen {
        return false;
    }
    for i in 0..slen {
        if s[i] != t[i] {
            return false;
        }
    }
    true
}

/// Look for a directory entry in a directory.
/// If found, set *poff to byte offset of entry.
pub fn dirlookup(dp_idx: usize, name: &[u8], poff: Option<&mut u32>) -> Option<usize> {
    unsafe {
        if dp_idx >= NINODE {
            panic!("dirlookup: bad inode index");
        }

        let dp = &ICACHE.inode[dp_idx];
        if (dp.type_ as u16) != T_DIR {
            panic!("dirlookup not DIR");
        }

        let mut off: u32 = 0;
        let de_size = DIRENT_SIZE as u32;

        while off < dp.size {
            let mut raw = [0u8; DIRENT_SIZE];
            if readi(dp_idx, &mut raw, off, de_size) != de_size as i32 {
                panic!("dirlookup read");
            }

            let de = parse_dirent(&raw);
            if de.inum == 0 {
                off += de_size;
                continue;
            }

            if namecmp(name, &de.name) {
                // entry matches path element
                if let Some(poff_ref) = poff {
                    *poff_ref = off;
                }
                let inum = de.inum as u32;
                return Some(iget(dp.dev, inum));
            }
            off += de_size;
        }

        None
    }
}

// Paths

/// Copy the next path element from path into name.
/// Return a pointer to the element following the copied one.
fn skipelem<'a>(path: &'a [u8], name: &mut [u8; DIRSIZ]) -> Option<&'a [u8]> {
    let mut i = 0;

    // Skip leading slashes
    while i < path.len() && path[i] == b'/' {
        i += 1;
    }
    if i >= path.len() || path[i] == 0 {
        return None;
    }

    let start = i;
    // Find end of this path element
    while i < path.len() && path[i] != b'/' && path[i] != 0 {
        i += 1;
    }
    let len = i - start;

    if len >= DIRSIZ {
        name.copy_from_slice(&path[start..start + DIRSIZ]);
    } else {
        name[..len].copy_from_slice(&path[start..start + len]);
        name[len] = 0;
        for j in (len + 1)..DIRSIZ {
            name[j] = 0;
        }
    }

    // Skip trailing slashes
    while i < path.len() && path[i] == b'/' {
        i += 1;
    }

    Some(&path[i..])
}

/// Look up and return the inode for a path name.
/// If nameiparent != 0, return the inode for the parent.
fn namex(path: &[u8], nameiparent_flag: bool, name: &mut [u8; DIRSIZ]) -> Option<usize> {
    let mut ip = iget(crate::param::ROOTDEV, ROOTINO);

    let mut remaining = path;
    loop {
        match skipelem(remaining, name) {
            None => break,
            Some(rest) => {
                iread(ip);
                unsafe {
                    if (ICACHE.inode[ip].type_ as u16) != T_DIR {
                        irelse(ip);
                        return None;
                    }
                }

                // If looking for parent and this is the last element
                if nameiparent_flag && (rest.is_empty() || rest[0] == 0) {
                    // Stop one level early.
                    return Some(ip);
                }

                match dirlookup(ip, name, None) {
                    None => {
                        irelse(ip);
                        return None;
                    }
                    Some(next) => {
                        irelse(ip);
                        ip = next;
                    }
                }
                remaining = rest;
            }
        }
    }

    if nameiparent_flag {
        irelse(ip);
        return None;
    }
    Some(ip)
}

/// Look up the inode for a path name.
pub fn namei(path: &[u8]) -> Option<usize> {
    let mut name = [0u8; DIRSIZ];
    namex(path, false, &mut name)
}

/// Look up the parent inode for a path name.
/// Copy the final path element into name.
pub fn nameiparent(path: &[u8], name: &mut [u8; DIRSIZ]) -> Option<usize> {
    namex(path, true, name)
}
