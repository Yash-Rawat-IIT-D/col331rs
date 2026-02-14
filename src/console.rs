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

pub fn consputc(c: char) {
    if c == BACKSPACE {
        uartputc(BACKSPACE);
        uartputc(' ');
        uartputc(BACKSPACE);
    } else {
        uartputc(c);
    }
}