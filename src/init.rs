use crate::user::{open, close, printf};
use crate::fcntl::O_RDWR;

#[no_mangle]
pub extern "C" fn main(argc: i32, argv: *const *const u8) -> i32 {
    unsafe {
        let fd = open(b"console\0".as_ptr(), O_RDWR);

        if argc > 0 {
            let arg0 = *argv;
            printf(fd, b"Hello %s from init.rs\n\0".as_ptr(), arg0);
        } else {
            printf(fd, b"Hello from init.rs\n\0".as_ptr());
        }

        close(fd);

        loop {}
    }
}