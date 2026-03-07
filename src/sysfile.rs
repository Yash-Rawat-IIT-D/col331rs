use crate::file;
use crate::param::NOFILE;
use crate::proc::myproc;
use crate::syscall::{argint, argstr};

fn argfd(n: i32, pfd: Option<&mut i32>, pf: Option<&mut usize>) -> i32 {
    let mut fd = 0;
    if argint(n, &mut fd) < 0 {
        return -1;
    }
    if fd < 0 || fd as usize >= NOFILE {
        return -1;
    }

    let curproc = match myproc() {
        Some(p) => p,
        None => return -1,
    };

    let fidx = match curproc.ofile[fd as usize] {
        Some(v) => v,
        None => return -1,
    };

    if let Some(out) = pfd {
        *out = fd;
    }
    if let Some(out) = pf {
        *out = fidx;
    }

    0
}

fn fdalloc(f_idx: usize) -> i32 {
    let curproc = match myproc() {
        Some(p) => p,
        None => return -1,
    };

    for fd in 0..NOFILE {
        if curproc.ofile[fd].is_none() {
            curproc.ofile[fd] = Some(f_idx);
            return fd as i32;
        }
    }

    -1
}

pub fn sys_write() -> i32 {
    let mut f_idx = 0usize;
    let mut p: *const u8 = core::ptr::null();

    let n = if argfd(0, None, Some(&mut f_idx)) < 0 {
        -1
    } else {
        argstr(1, &mut p)
    };

    if n < 0 {
        return -1;
    }

    let s = unsafe { core::slice::from_raw_parts(p, n as usize) };
    file::filewrite(f_idx, s, n)
}

pub fn sys_close() -> i32 {
    let mut fd = 0;
    let mut f_idx = 0usize;

    if argfd(0, Some(&mut fd), Some(&mut f_idx)) < 0 {
        return -1;
    }

    if let Some(curproc) = myproc() {
        curproc.ofile[fd as usize] = None;
    }
    file::fileclose(f_idx);
    0
}

pub fn sys_open() -> i32 {
    let mut path_ptr: *const u8 = core::ptr::null();
    let mut omode = 0;

    let path_len = argstr(0, &mut path_ptr);
    if path_len < 0 || argint(1, &mut omode) < 0 {
        return -1;
    }

    let path_bytes = unsafe { core::slice::from_raw_parts(path_ptr, path_len as usize) };
    let path = match core::str::from_utf8(path_bytes) {
        Ok(v) => v,
        Err(_) => return -1,
    };

    let f_idx = match file::open(path, omode) {
        Some(v) => v,
        None => return -1,
    };

    let fd = fdalloc(f_idx);
    if fd < 0 {
        file::fileclose(f_idx);
    }
    fd
}
