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
mod buf;
mod bio;
mod ide;
mod fs;
mod fcntl;
mod file;
mod log;
mod mmu;
mod vm;
use crate::traps::*;

fn halt() -> ! {
    println!("Bye COL{}\n\0", 331);
    loop {
        x86::outw(0x602, 0x2000);
        x86::outw(0xB002, 0x2000);
    }
}

fn welcome() {
    // Use println! to verify we reach this point (goes via Console::Write, not file)
   
    let c = match file::open("/console", fcntl::O_RDWR) {
        Some(fd) => {
            fd
        }
        None => {
            panic!("Failed to open console");
        }
    };

    let enter_message = b"\nEnter your name: ";
    file::filewrite(c, enter_message, enter_message.len() as i32);
    
    let mut name = [0u8; 20];
    let nice_message = b"Nice to meet you! ";
    let bye_message = b"BYE!\n";
    let namelen = file::fileread(c, &mut name, 20);
    file::filewrite(c, nice_message, nice_message.len() as i32);
    file::filewrite(c, &name[..namelen as usize], namelen);
    file::filewrite(c, bye_message, bye_message.len() as i32); // Goodbye message is 5 bytes not 6 (Rust vs C string handling)
    
    file::fileclose(c);
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
    console::consoleinit();
    uart::uartinit();
    ide::ideinit();
    tvinit();
    bio::binit();
    idtinit();
    x86::sti();
    fs::iinit(param::ROOTDEV);
    log::initlog(param::ROOTDEV);
    file::mknod("/console", param::CONSOLE as i16, param::CONSOLE as i16);
    vm::seginit();       // segment descriptors
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
