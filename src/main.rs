#![no_std]
#![no_main]
use core::arch::asm;

#[allow(dead_code)]
fn outw(port: u16, data: u16) {
  unsafe {
    asm!("out dx, ax", in("dx") port, in("ax") data);
  }
}

fn halt() -> ! {
  outw(0x604, 0x2000);
  // For older versions of QEMU, 
  outw(0xB004, 0x2000);
  loop {}
}

#[no_mangle]
fn entryofrust() -> ! {
  halt();
}

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}