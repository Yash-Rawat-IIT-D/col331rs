use core::cell::RefCell;
use core::mem::size_of;
use core::sync::atomic::Ordering;

use crate::{bio, fs_h};
use crate::buf::{B_DIRTY, BSIZE};
use crate::fs;
use crate::param::LOGSIZE;

#[derive(Copy, Clone)]
struct LogHeader {
    n: i32,
    block: [u32; LOGSIZE],
}

impl LogHeader {
    const fn new() -> Self {
        Self {
            n: 0,
            block: [0; LOGSIZE],
        }
    }
}

struct Log {
    start: u32,
    size: u32,
    committing: bool,
    dev: u32,
    lh: LogHeader,
}

impl Log {
    const fn new(start: u32, size: u32, dev: u32) -> Self {
        Self {
            start,
            size,
            committing: false,
            dev,
            lh: LogHeader::new(),
        }
    }
}

// static mut LOG: RefCell<Log> = RefCell::new(Log::new());

#[inline]
fn read_i32_le(data: &[u8], off: usize) -> i32 {
    i32::from_le_bytes([data[off], data[off + 1], data[off + 2], data[off + 3]])
}

#[inline]
fn write_i32_le(data: &mut [u8], off: usize, val: i32) {
    data[off..off + 4].copy_from_slice(&val.to_le_bytes());
}

#[inline]
fn read_u32_le(data: &[u8], off: usize) -> u32 {
    u32::from_le_bytes([data[off], data[off + 1], data[off + 2], data[off + 3]])
}

#[inline]
fn write_u32_le(data: &mut [u8], off: usize, val: u32) {
    data[off..off + 4].copy_from_slice(&val.to_le_bytes());
}

impl Log {
    fn read_head(self: &mut Self) {
        let buf = bio::bread(self.dev, self.start);
        let data = &bio::buf_mut(buf).data;
        self.lh.n = read_i32_le(data, 0);
        for i in 0..(self.lh.n as usize) {
            self.lh.block[i] = read_u32_le(data, 4 + i * 4);
        }
        bio::brelse(buf);
    }

    fn write_head(self: &Self) {
        let buf = bio::bread(self.dev, self.start);
        let data = &mut bio::buf_mut(buf).data;
        data.fill(0);
        write_i32_le(data, 0, self.lh.n);
        for i in 0..(self.lh.n as usize) {
            write_u32_le(data, 4 + i * 4, self.lh.block[i]);
        }
        bio::bwrite(buf);
        bio::brelse(buf);
    }

    fn install_trans(self: &Self) {
        for tail in 0..(self.lh.n as usize) {
            let lbuf = bio::bread(self.dev, self.start + tail as u32 + 1);
            let dbuf = bio::bread(self.dev, self.lh.block[tail]);
            let src = bio::buf_mut(lbuf).data;
            bio::buf_mut(dbuf).data.copy_from_slice(&src);
            bio::bwrite(dbuf);
            bio::brelse(lbuf);
            bio::brelse(dbuf);
        }
    }

    fn recover_from_log(self: &mut Self) {
        self.read_head();
        self.install_trans();
        self.lh.n = 0;
        self.write_head();
    }

    fn write_log(self: &Self) {
        for tail in 0..(self.lh.n as usize) {
            let to = bio::bread(self.dev, self.start + tail as u32 + 1);
            let from = bio::bread(self.dev, self.lh.block[tail]);
            let src = bio::buf_mut(from).data;
            bio::buf_mut(to).data.copy_from_slice(&src);
            bio::bwrite(to);
            bio::brelse(from);
            bio::brelse(to);
        }
    }

    fn commit(self: &mut Self) {
        if self.committing {
            panic!("commit");
        }
        self.committing = true;
        if self.lh.n > 0 {
            self.write_log();
            self.write_head();
            self.install_trans();
            self.lh.n = 0;
            self.write_head();
        }
        self.committing = false;
    }

    pub fn initlog(dev: u32) -> Self {
        if size_of::<LogHeader>() >= BSIZE {
            panic!("initlog: too big logheader");
        }

        let mut sb = fs_h::Superblock::new();
        fs::readsb(dev, &mut sb);
        let mut log = Log::new(sb.logstart, sb.nlog, dev);
        log.recover_from_log();
        log
    }

    pub fn log_write(self: &mut Self, idx: usize) {
        let blockno = bio::buf_mut(idx).blockno;
        if self.lh.n as usize >= LOGSIZE || self.lh.n as u32 >= self.size.saturating_sub(1) {
            panic!("too big a transaction");
        }

        let mut i = 0usize;
        while i < self.lh.n as usize {
            if self.lh.block[i] == blockno {
                break;
            }
            i += 1;
        }

        self.lh.block[i] = blockno;
        if i == self.lh.n as usize {
            self.lh.n += 1;
        }

        bio::buf_mut(idx).flags.fetch_or(B_DIRTY, Ordering::AcqRel);
    }
}


pub fn begin_op(log: &mut Log) {}

pub fn end_op(log: &mut Log) {
    log.commit();
}


