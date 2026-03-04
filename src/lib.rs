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
