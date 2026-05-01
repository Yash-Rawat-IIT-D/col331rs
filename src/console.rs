use crate::uart::*;
use core::fmt::*;

pub struct Console {}
impl Write for Console {
    fn write_str(&mut self, s: &str) -> Result {
         for c in s.chars() {
            consputc(c as u8);
        }
        Ok(())
    }
}

const BACKSPACE: u8 = '\x08' as u8;
const INPUT_BUF: usize = 128;

const fn ctrl(x: char) -> u8 {  // Control-x
    (x as u8) - ('@' as u8)
}

#[derive(Clone, Copy)]
struct Input {
    buf: [u8; INPUT_BUF],
    r: usize,
    w: usize,
    e: usize,
}

static mut INPUT: Input = Input {
    buf: [0; INPUT_BUF],
    r: 0,
    w: 0,
    e: 0,
};

pub fn consoleintr(getc: fn() -> Option<u8>) {
    loop {
        let c_opt = getc();
        let input = unsafe { &mut INPUT };

        if let Some(c) = c_opt {
            match c {
                x if x == ctrl('U') => {
                    while input.e != input.w && input.buf[(input.e - 1) % INPUT_BUF] != b'\n' {
                        input.e -= 1;
                        consputc(BACKSPACE);
                    }
                }
                x if x == ctrl('H') || x == 0x7f => {
                    if input.e != input.w {
                        input.e -= 1;
                        consputc(BACKSPACE);
                    }
                }
                _ => {
                    if c != 0 && input.e.wrapping_sub(input.r) < INPUT_BUF {
                        let c = if c == b'\r' { b'\n' } else { c };
                        input.buf[input.e % INPUT_BUF] = c as u8;
                        input.e += 1;
                        consputc(c);
                        if c == b'\n' || c == ctrl('D') || input.e == input.r + INPUT_BUF {
                            input.w = input.e;
                        }
                    }
                }
            }
        }
    }
}

#[macro_export]
macro_rules! println {
    ($($arg:tt)*) => ({
        use core::fmt::*;
        use crate::console::Console;
        let mut c = Console {};
        let _ = writeln!(&mut c, $($arg)*);
    });
}

fn consputc(c: u8) {
    if c == BACKSPACE {
        uartputc(BACKSPACE as char);
        uartputc(' ');
        uartputc(BACKSPACE as char);
    } else {
        uartputc(c as char);
    }
}