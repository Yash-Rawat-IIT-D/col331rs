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
    let root = fs::namei("/").unwrap_or_else(|| panic!("root not found"));
    fs::iread(root);
    let foodir = match fs::dirlookup(root, "foo", None) {
        Some(idx) => idx,
        None => {
            log::begin_op();
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
            log::end_op();
            idx
        }
    };

    let wtxt = match fs::namei("/foo/greet.txt") {
        Some(idx) => idx,
        None => {
            log::begin_op();
            println!("/foo/greet.txt not found. Creating!");
            let wtxt_orig =
                fs::namei("/welcome.txt").unwrap_or_else(|| panic!("/welcome.txt missing"));
            let inum = fs::inode_inum(wtxt_orig);
            if fs::dirlink(foodir, "greet.txt", inum) < 0 {
                panic!("failed to link greet.txt in /foo");
            }
            fs::irelease(wtxt_orig);
            log::end_op();
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
    console::consputc('\n');

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
    log::initlog(param::ROOTDEV);
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
