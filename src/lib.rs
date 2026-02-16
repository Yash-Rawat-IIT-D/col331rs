#![no_std]       // No standard library
#![no_main]      // No main function

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

fn halt() -> ! {
    println!("Bye COL{}\n\0", 331);
    loop {
        x86::outw(0x602, 0x2000);
        x86::outw(0xB002, 0x2000);
    }
}

extern "C" {
    pub static alltraps: fn();
}

#[no_mangle]
pub extern "C" fn entryofrust() -> ! {
    mp::mpinit();
    lapic::lapicinit();
    picirq::picinit();
    ioapic::ioapic_init();
    uart::uartinit();
    tvinit();
    idtinit();
    x86::sti();
    loop {
        x86::wfi();
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("Kernel Panic: {:?}", info);
    halt()
}