# COL331RS: p11-file-layer (Rust)

This branch ports the `p11-file-layer` stage from the C repo (`codenet/col331`) into Rust in `codenet/col331rs` style.

## What This Branch Adds

- Rust file descriptor layer (`src/file.rs`):
  - `filealloc`, `filedup`, `fileclose`, `filestat`, `fileread`, `filewrite`
  - pathname/file ops: `open`, `create`, `unlink`, `isdirempty`
- Open-mode constants (`src/fcntl.rs`):
  - `O_RDONLY`, `O_WRONLY`, `O_RDWR`, `O_CREATE`
- File-system lifecycle changes in `src/fs.rs`:
  - block free path (`bfree`)
  - inode truncation (`itrunc`)
  - reference-drop semantics (`iput`) matching p11 behavior
  - updated read/path behavior to work with unlink + final `iput`
- Rust `mkfs` integration (`src/mkfs.rs`):
  - no dependency on external `../col331/mkfs.c`
- Boot demo update (`src/lib.rs`):
  - creates `/foo/hello.txt`, writes and reads it, unlinks it, verifies directory emptiness
  - reads `/welcome.txt` through the file layer

## Build And Run

Make sure your cross-toolchain is available (for example `TOOLPREFIX=i686-elf-`).

```bash
make clean
make xv6.img fs.img
make qemu
```

## Source Alignment

- C reference branch: `codenet/col331` -> `p11-file-layer`
- Rust base used: p10 Rust work, then extended to p11 file layer behavior

## Credits

This branch reuses and builds on earlier COL331RS community work:

- **Amber-Agarwal** (`refs/pull/7/head` in `codenet/col331rs`)
  - p10-level Rust filesystem write path and directory/name operation groundwork
- **Nipun Goel** (aka **sudoheckbeluga**, `refs/pull/5/head`)
  - early Rust `mkfs` implementation used as the basis for later versions
- **legends1307** (`refs/pull/12/head`)
  - self-contained Rust `mkfs` adaptation (`FSSIZE=1000`, `LOGSIZE=0`) and related integration improvements
- **COL331 course/staff repository authors** (`codenet/col331`, `codenet/col331rs`)
  - original xv6 step-by-step structure and assignment progression

