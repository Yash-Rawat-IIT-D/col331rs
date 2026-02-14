use crate::x86::{outb, inb};
use crate::console::consoleintr;
use crate::ioapic::ioapic_enable;
use core::sync::atomic::{AtomicBool, Ordering};
const COM1: u16 = 0x3F8; // COM1 port address
pub const IRQ_COM1: u32 = 4;

static UART_PRESENT: AtomicBool = AtomicBool::new(false);

pub fn uartinit() {
    // Turn off the FIFO.
    outb(COM1 + 2, 0);

    // Set 9600 baud, 8 data bits, 1 stop bit, parity off.
    outb(COM1 + 3, 0x80); // Unlock divisor.
    let b = 115200 / 9600;
    outb(COM1 + 0, b as u8);
    outb(COM1 + 1, 0);
    outb(COM1 + 3, 0x03); // Lock divisor, 8 data bits.
    outb(COM1 + 4, 0);
    outb(COM1 + 1, 0x01); // Enable receive interrupts.

    // If status is 0xFF, no serial port is present.
    if inb(COM1 + 5) == 0xFF {
        return;
    }

    UART_PRESENT.store(true, Ordering::Relaxed);

    // Acknowledge pre-existing interrupt conditions;
    // enable interrupts.
    inb(COM1+2);
    inb(COM1+0);
    ioapic_enable(IRQ_COM1, 0);

    // Announce that the UART is active.
    for p in "xv6...\n".chars() {
        uartputc(p);
    }
}

pub fn uartputc(c: char) {
    if !UART_PRESENT.load(Ordering::Relaxed) {
        return;
    }
    for _ in 0..128 {
        if inb(COM1 + 5) & 0x20 != 0 {
            break;
        }
    }
    outb(COM1 + 0, c as u8);
}

fn uartgetc() -> i32 {
    if !UART_PRESENT.load(Ordering::Relaxed) {
        return -1;
    }
    if (inb(COM1 + 5) & 0x01) == 0 {
        return -1;
    }
    inb(COM1 + 0) as i32
}

pub fn uartintr() {
    consoleintr(uartgetc);
}
