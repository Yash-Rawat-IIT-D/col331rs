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
mod constants;  
mod buf;
mod bio;
mod ide;
mod fs;
mod fcntl;
mod file;
mod log;
use crate::traps::*;

#[macro_export]
macro_rules! println {
    ($($arg:tt)*) => ({
        use core::fmt::Write;
        use crate::console::*;
        let mut console = Console {};
        let _ = writeln!(&mut console, $($arg)*);
    });
}

fn halt() -> ! {
    println!("Bye COL{}\n\0", 331);
    loop {
        x86::outw(0x604, 0x2000);  // QEMU isa-debug-exit device
        x86::outw(0xB004, 0x2000); // VirtualBox shutdown port
    }
}

fn print_cstr(bytes: &[u8]) {
    for &ch in bytes {
        if ch == 0 {
            break;
        }
        console::consputc(ch as i32);
    }
}

fn welcome() {
    // Create and write /foo/hello.txt
    file::mkdir("/foo");
    
    let gtxt = file::open("/foo/hello.txt", fcntl::O_CREATE | fcntl::O_WRONLY)
        .unwrap_or_else(|| panic!("Failed to create /foo/hello.txt"));
    let n = file::filewrite(gtxt, b"hello\0", 6);
    println!("Wrote {} characters to /foo/hello.txt", n);
    file::fileclose(gtxt);
    
    let gtxt = file::open("/foo/hello.txt", fcntl::O_RDONLY)
        .unwrap_or_else(|| panic!("Unable to open /foo/hello.txt"));
    let mut welcome = [0u8; 512];
    let n = file::fileread(gtxt, &mut welcome, 6);
    println!("Read {} chars from /foo/hello.txt: ", n);
    print_cstr(&welcome);
    console::consputc('\n' as i32);
    file::fileclose(gtxt);
    
    // Delete /foo/hello.txt
    let mut name = [0u8; constants::DIRSIZ];
    file::unlink("/foo/hello.txt", &mut name);
    
    let foo = fs::namei("/foo").unwrap_or_else(|| panic!("unable to open /foo"));
    if !file::isdirempty(foo) {
        panic!("/foo should be empty");
    }
    
    if let Some(_gtxt) = file::open("/foo/hello.txt", fcntl::O_RDONLY) {
        panic!("Could open /foo/hello.txt after unlinking");
    }
    
    // Print welcome message
    let wtxt = file::open("/welcome.txt", fcntl::O_RDONLY)
        .unwrap_or_else(|| panic!("Unable to open /welcome.txt"));
    let n = file::fileread(wtxt, &mut welcome, 512);
    println!("Read {} chars from /welcome.txt:\n", n);
    print_cstr(&welcome);
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
    log::initlog(param::ROOTDEV);
    welcome();

    loop {
        x86::wfi();
    }
}

static mut PANICKED: bool = false;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    // Disable interrupts to prevent interrupt handlers from interfering
    cli();
    // Print panic message with LAPIC ID to identify which CPU panicked
    println!("lapicid {}:\n{:#?}", lapicid(), info);
    unsafe { PANICKED = true; }
    // Halt the system
    loop {}
}
