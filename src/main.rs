#![no_std]       // No standard library
#![no_main]      // No main function
#![allow(dead_code)]

use core::panic::PanicInfo;
use crate::constants::{DIRENT_SIZE, DIRSIZ};
use crate::x86::cli;
use crate::lapic::lapicid;

mod param;
mod x86;
mod uart;
mod console;
mod lapic;
mod ioapic;
mod picirq;
mod mp;
mod proc;
mod traps;
mod fs_h;
mod constants;  // Internal use only - no external crates
mod buf;
mod bio;
mod ide;
mod fs;
mod file;
mod fcntl;
use crate::traps::*;

fn halt() -> ! {
    println!("Bye COL{}\n\0", 331);
    loop {
        x86::outw(0x602, 0x2000);
        x86::outw(0xB002, 0x2000);
    }
}

fn welcome() {
    // Create /foo
    let _ = file::create("/foo", constants::T_DIR as i16, 0, 0);

    // Write /foo/hello.txt
    let gtxt = file::open("/foo/hello.txt", fcntl::O_CREATE | fcntl::O_WRONLY)
        .unwrap_or_else(|| panic!("failed to create /foo/hello.txt"));
    let n = file::filewrite(gtxt, b"hello\0", 6);
    println!("Wrote {} characters to /foo/hello.txt", n);
    file::fileclose(gtxt);

    // Read /foo/hello.txt
    let gtxt = file::open("/foo/hello.txt", fcntl::O_RDONLY)
        .unwrap_or_else(|| panic!("unable to open /foo/hello.txt"));
    let mut welcome = [0u8; 512];
    let n = file::fileread(gtxt, &mut welcome, 6);
    println!("Read {} chars from /foo/hello.txt: ", n);
    println!("{}\n", core::str::from_utf8(&welcome).unwrap_or("?"));
    file::fileclose(gtxt);

    // Delete /foo/hello.txt
    let mut name: [u8; DIRSIZ] = [0; DIRSIZ];
    name[..9].copy_from_slice(b"hello.txt");
    if file::unlink("/foo/", name) < 0 {
        panic!("failed to unlink /foo/hello.txt");
    }

    // Check that /foo is empty
    let foo = fs::namei("/foo").unwrap_or_else(|| panic!("unable to open /foo"));
    if !file::isdirempty(foo) {
        panic!("/foo should be empty");
    }
    fs::iput(foo);

    // Check that we cannot read file /foo/hello.txt
    if let Some(f) = file::open("/foo/hello.txt", fcntl::O_RDONLY) {
        file::fileclose(f);
        panic!("could open /foo/hello.txt after unlinking");
    }

    // Write to /welcome.txt
    let wtxt =
        file::open("/welcome.txt", fcntl::O_RDONLY).unwrap_or_else(|| panic!("unable to open /welcome.txt"));
    let welcome_cap = welcome.len() as i32;
    let n = file::fileread(wtxt, &mut welcome, welcome_cap);
    println!("Read {} chars from /welcome.txt:", n);
    println!("{}\n", core::str::from_utf8(&welcome).unwrap_or("?"));
    file::fileclose(wtxt);
}

extern "C" {
    pub fn alltraps();
}

#[no_mangle]
pub extern "C" fn entryofrust() -> ! {
    mp::mpinit();
    lapic::lapicinit();
    picirq::picinit();
    ioapic::ioapic_init();
    uart::uartinit();
    ide::ideinit();
    tvinit();
    bio::binit();
    idtinit();
    x86::sti();
    fs::iinit(param::ROOTDEV);
    file::fileinit();
    welcome();

    loop {
        x86::wfi();
    }
}

#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("Kernel Panic: {:?}", info);
    halt()
}
