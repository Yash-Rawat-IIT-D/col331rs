#![no_std]       // No standard library
#![no_main]      // No main function

use core::panic::PanicInfo;

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
use crate::traps::*;

fn halt() -> ! {
    println!("Bye COL{}\n\0", 331);
    loop {
        x86::outw(0x602, 0x2000);
        x86::outw(0xB002, 0x2000);
    }
}

fn welcome() {
    let b0 = bio::bread(1, 0);
    let data0 = bio::buf_mut(b0).data;

    for &byte in data0.iter() {
        if byte == 0 { break; }
        console::consputc(byte);
    }
    bio::brelse(b0);

    let b1 = bio::bread(1, 1);
    let count = bio::buf_mut(b1).data[0];

    println!("\nAfter preparing fs.img, we have rebooted {} times\n", count);

    bio::buf_mut(b1).data[0] = count.wrapping_add(1);
    bio::bwrite(b1);
    bio::brelse(b1);
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