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
mod constants;  // Internal use only - no external crates
mod fs;         // Internal use only - filesystem structures
mod buf;
mod bio;
mod ide;
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

fn print_bytes(bytes: &[u8]) {
    for &ch in bytes {
        console::consputc(ch as i32);
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
    let root = fs::namei("/").unwrap_or_else(|| panic!("root not found"));
    fs::iread(root);
    let foodir = match fs::dirlookup(root, "foo", None) {
        Some(idx) => idx,
        None => {
            println!("/foo not found. Creating!");
            let idx = fs::ialloc(param::ROOTDEV, fs::T_DIR);
            fs::iread(idx);
            let ino = fs::inode_inum(idx);
            if fs::dirlink(idx, ".", ino) < 0 {
                panic!("failed to link . in /foo");
            }
            if fs::dirlink(idx, "..", ino) < 0 {
                panic!("failed to link .. in /foo");
            }
            if fs::dirlink(root, "foo", ino) < 0 {
                panic!("failed to link /foo in root");
            }
            idx
        }
    };

    let wtxt = match fs::namei("/foo/greet.txt") {
        Some(idx) => idx,
        None => {
            println!("/foo/greet.txt not found. Creating!");
            let wtxt_orig =
                fs::namei("/welcome.txt").unwrap_or_else(|| panic!("/welcome.txt missing"));
            let inum = fs::inode_inum(wtxt_orig);
            if fs::dirlink(foodir, "greet.txt", inum) < 0 {
                panic!("failed to link greet.txt in /foo");
            }
            fs::irelease(wtxt_orig);
            fs::namei("/foo/greet.txt").unwrap_or_else(|| panic!("greet.txt lookup failed"))
        }
    };

    fs::iread(wtxt);
    let mut st = fs::Stat::new();
    fs::stati(wtxt, &mut st);

    let mut greet = [0u8; 512];
    let n = fs::readi(wtxt, &mut greet, 0, st.size);
    println!("Read {} bytes from /foo/greet.txt", n);
    print_bytes(&greet[..n as usize]);
    console::consputc('\n' as i32);

    fs::irelease(wtxt);
    fs::irelease(foodir);
    fs::irelease(root);
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
