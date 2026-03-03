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
    const NDIR_READ: usize = 4;

    let root = fs::iget(param::ROOTDEV, fs::ROOTINO);
    fs::iread(root);

    let mut raw_entries = [0u8; fs::DIRENT_SIZE * NDIR_READ];
    let entries_len = raw_entries.len() as u32;
    let n = fs::readi(root, &mut raw_entries, 0, entries_len);
    println!("Read {} bytes from inode of root directory", n);

    let mut entries = [fs::Dirent::new(); NDIR_READ];
    for i in 0..NDIR_READ {
        let start = i * fs::DIRENT_SIZE;
        let end = start + fs::DIRENT_SIZE;
        entries[i] = fs::parse_dirent(&raw_entries[start..end]);

        print_bytes(b"name: ");
        print_cstr(&entries[i].name);
        println!(" is at inum: {}", entries[i].inum);
    }

    let wtxt = fs::iget(param::ROOTDEV, entries[2].inum as u32);
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
    print_bytes(&greet[..n as usize]);
    console::consputc('\n' as i32);
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
