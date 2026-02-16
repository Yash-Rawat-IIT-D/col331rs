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
mod fs;
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
        x86::outw(0x604, 0x2000);
        x86::outw(0xB004, 0x2000);
    }
}

fn print_bytes(bytes: &[u8]) {
    for &ch in bytes {
        console::consputc(ch as char);
    }
}

fn print_cstr(bytes: &[u8]) {
    for &ch in bytes {
        if ch == 0 {
            break;
        }
        console::consputc(ch as char);
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
    print_bytes(&greet[..n as usize]);
    console::consputc('\n');
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

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("Kernel Panic: {:?}", info);
    loop {}
}
