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


#[macro_export]
macro_rules! println {
    ($($arg:tt)*) => ({
        use core::fmt::*;
        use crate::console::Console;
        let mut c = Console {};
        let _ = writeln!(&mut c, $($arg)*);
    });
}

const BACKSPACE: char = '\x08';

fn consputc(c: char) {
  if c == BACKSPACE {
    uartputc(BACKSPACE);
    uartputc(' ');
    uartputc(BACKSPACE);
  } else {
    uartputc(c);
  }
}
