extern "C" {
    pub fn write(fd: i32, buf: *const u8, n: i32) -> i32;
    pub fn close(fd: i32) -> i32;
    pub fn open(path: *const u8, flags: i32) -> i32;
    pub fn exec(path: *const u8) -> i32;
}