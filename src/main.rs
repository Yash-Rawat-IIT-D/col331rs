#![no_std]       // No standard library
#![no_main]      // No main function
#![allow(dead_code)]

use core::panic::PanicInfo;
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
mod fs;         // Internal use only - filesystem structures
mod buf;
mod bio;
mod ide;
use crate::traps::*;

fn halt() -> ! {
    println!("Bye COL{}\n\0", 331);
    loop {
        x86::outw(0x602, 0x2000);
        x86::outw(0xB002, 0x2000);
    }
}

fn welcome() {
    // p9: Use namei to look up /welcome.txt by path instead of manually
    // reading directory entries from the root inode.
    let wtxt = fs::namei(b"/welcome.txt").unwrap_or_else(|| {
        panic!("welcome: /welcome.txt not found");
    });
    fs::iread(wtxt);
    let mut st = fs::Stat::new();
    fs::stati(wtxt, &mut st);
    println!(
        "\nwelcome.txt stats: Device {}, inode number {}, type {}, number of links {}, size {}",
        st.dev, st.ino, st.type_, st.nlink, st.size
    );

    let mut greet = [0u8; 512];
    let n = fs::readi(wtxt, &mut greet, 0, st.size);
    println!("Read {} bytes from welcome.txt", n);
    println!("{}\n", core::str::from_utf8(&greet[..n as usize]).unwrap_or("?"));
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
