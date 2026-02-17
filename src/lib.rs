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
mod file;
mod fcntl;
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

fn print_cstr(bytes: &[u8]) {
    for &ch in bytes {
        if ch == 0 {
            break;
        }
        console::consputc(ch as char);
    }
}

fn welcome() {
    let _ = file::create("/foo", fs::T_DIR, 0, 0);

    let gtxt = file::open("/foo/hello.txt", fcntl::O_CREATE | fcntl::O_WRONLY)
        .unwrap_or_else(|| panic!("failed to create /foo/hello.txt"));
    let n = file::filewrite(gtxt, b"hello\0", 6);
    println!("Wrote {} characters to /foo/hello.txt", n);
    file::fileclose(gtxt);

    let gtxt = file::open("/foo/hello.txt", fcntl::O_RDONLY)
        .unwrap_or_else(|| panic!("unable to open /foo/hello.txt"));
    let mut welcome = [0u8; 512];
    let n = file::fileread(gtxt, &mut welcome, 6);
    println!("Read {} chars from /foo/hello.txt: ", n);
    print_cstr(&welcome);
    console::consputc('\n');
    file::fileclose(gtxt);

    let mut name = [0u8; fs::DIRSIZ];
    if file::unlink("/foo/hello.txt", &mut name) < 0 {
        panic!("failed to unlink /foo/hello.txt");
    }

    let foo = fs::namei("/foo").unwrap_or_else(|| panic!("unable to open /foo"));
    if !file::isdirempty(foo) {
        panic!("/foo should be empty");
    }
    fs::iput(foo);

    if let Some(f) = file::open("/foo/hello.txt", fcntl::O_RDONLY) {
        file::fileclose(f);
        panic!("could open /foo/hello.txt after unlinking");
    }

    let wtxt =
        file::open("/welcome.txt", fcntl::O_RDONLY).unwrap_or_else(|| panic!("unable to open /welcome.txt"));
    let welcome_cap = welcome.len() as i32;
    let n = file::fileread(wtxt, &mut welcome, welcome_cap);
    println!("Read {} chars from /welcome.txt:", n);
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
    file::fileinit();
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
