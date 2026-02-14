use crate::uart::*;
use core::fmt::*;
pub struct Console {}

impl Write for Console {
    fn write_str(&mut self, s: &str) -> Result {
        for c in s.chars() {
            consputc(c);
        }
        Ok(())
    }
}

const BACKSPACE: char = '\x08';
const INPUT_BUF: u32 = 128;

struct Input {
    buf: [u8; INPUT_BUF as usize],
    r: u32, // Read index
    w: u32, // Write index
    e: u32, // Edit index
}

static mut INPUT: Input = Input {
    buf: [0; INPUT_BUF as usize],
    r: 0,
    w: 0,
    e: 0,
};

const fn ctrl(x: u8) -> u8 {
    x - b'@'
}

pub fn consputc(c: char) {
    if c == BACKSPACE {
        uartputc(BACKSPACE);
        uartputc(' ');
        uartputc(BACKSPACE);
    } else {
        uartputc(c);
    }
}

pub fn consoleintr(getc: fn() -> i32) {
    loop {
        let ch_raw = getc();
        if ch_raw < 0 {
            break;
        }

        unsafe {
            match ch_raw as u8 {
                x if x == ctrl(b'U') => {
                    while INPUT.e != INPUT.w
                        && INPUT.buf[((INPUT.e - 1) % INPUT_BUF) as usize] != b'\n'
                    {
                        INPUT.e -= 1;
                        consputc(BACKSPACE);
                    }
                }
                x if x == ctrl(b'H') || x == 0x7f => {
                    if INPUT.e != INPUT.w {
                        INPUT.e -= 1;
                        consputc(BACKSPACE);
                    }
                }
                mut ch => {
                    if ch != 0 && INPUT.e - INPUT.r < INPUT_BUF {
                        if ch == b'\r' {
                            ch = b'\n';
                        }
                        INPUT.buf[(INPUT.e % INPUT_BUF) as usize] = ch;
                        INPUT.e += 1;
                        consputc(ch as char);
                        if ch == b'\n' || ch == ctrl(b'D') || INPUT.e == INPUT.r + INPUT_BUF {
                            INPUT.w = INPUT.e;
                        }
                    }
                }
            }
        }
    }
}
