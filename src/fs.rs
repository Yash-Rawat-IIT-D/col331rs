use core::cmp::min;

use crate::bio;
use crate::buf::BSIZE;
use crate::param::NINODE;
use crate::println;

pub const ROOTINO: u32 = 1;
pub const NDIRECT: usize = 12;
pub const NINDIRECT: usize = BSIZE / core::mem::size_of::<u32>();
pub const DIRSIZ: usize = 14;
pub const DIRENT_SIZE: usize = 2 + DIRSIZ;

const DINODE_SIZE: usize = 2 + 2 + 2 + 2 + 4 + ((NDIRECT + 1) * 4);
const IPB: u32 = (BSIZE / DINODE_SIZE) as u32;

#[derive(Copy, Clone)]
#[repr(C)]
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

#[derive(Copy, Clone)]
#[repr(C)]
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
    inum / IPB + sb.inodestart
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
        readsb(dev, &mut SB);
        println!(
            "sb: size {} nblocks {} ninodes {} nlog {} logstart {} inodestart {} bmap start {}",
            SB.size, SB.nblocks, SB.ninodes, SB.nlog, SB.logstart, SB.inodestart, SB.bmapstart
        );
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
            let bp = bio::bread(ip.dev, iblock(ip.inum, &SB));
            let data = &bio::buf_mut(bp).data;

            let off = (ip.inum % IPB) as usize * DINODE_SIZE;
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