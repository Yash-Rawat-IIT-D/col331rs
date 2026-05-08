use crate::{uart::*};
use core::fmt::*;
use crate::file::DEVSW;
use crate::param::CONSOLE;

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
    let mut doprocdump = false;
    loop {
        let c_opt = getc();
        let input = unsafe { &mut INPUT };

        if let Some(c) = c_opt {
            match c {
                x if x == ctrl('P') => {
                    // procdump() may indirectly use console output; call after loop
                    doprocdump = true;
                }
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
                        input.buf[input.e % INPUT_BUF] = c;
                        input.e += 1;
                        consputc(c);
                        if c == b'\n' || c == ctrl('D') || input.e == input.r + INPUT_BUF {
                            input.w = input.e;
                        }
                    }
                }
            }
        } else {
            break;
        }
    }
    if doprocdump {
        crate::proc::procdump();
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

pub fn consputc(c: u8) {
    if c == BACKSPACE {
        uartputc(BACKSPACE as char);
        uartputc(' ');
        uartputc(BACKSPACE as char);
    } else {
        uartputc(c as char);
    }
}

pub fn consoleread(_ip: usize, dst: &mut [u8], n: i32) -> i32 {
    let target = n;
    let mut n = n;

    unsafe {
        let input = &raw mut INPUT;
        while n > 0 {
            // Busy wait for input - mirrors C: while(input.r == input.w);
            while core::ptr::read_volatile(&(*input).r) == core::ptr::read_volatile(&(*input).w) {
                // Spin-wait (busy wait) for input to arrive
                core::hint::spin_loop();
            }

            // Read character and increment read pointer - mirrors C: input.buf[input.r++ % INPUT_BUF]
            let c = (*input).buf[(*input).r % INPUT_BUF];
            (*input).r += 1;
            
            // Handle EOF (Ctrl-D)
            if c == ctrl('D') {
                if n < target {
                    // Save ^D for next time, to make sure
                    // caller gets a 0-byte result.
                    (*input).r -= 1;
                }
                break;
            }
            
            // Copy character to destination - mirrors C: *dst++ = c;
            dst[(target - n) as usize] = c as u8;
            n -= 1;
            
            // Break on newline
            if c == b'\n' {
                break;
            }
        }
    }

    target - n
}

pub fn consolewrite(_ip: usize, src: &[u8], n: i32) -> i32 {
    // Mirrors C: for(i = 0; i < n; i++) consputc(buf[i] & 0xff);
    for i in 0..n {
        consputc(src[i as usize]);
    }
    n
}

pub fn consoleinit() {
    // Register console device handlers in the device switch table
    // Mirrors C: devsw[CONSOLE].write = consolewrite; devsw[CONSOLE].read = consoleread;
    unsafe {
        DEVSW[CONSOLE].read  = Some(consoleread);
        DEVSW[CONSOLE].write = Some(consolewrite);
    }
}
