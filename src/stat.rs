use constants::*;

#[repr(C)]
pub struct Stat {
    pub type_: i16,   // Type of file
    pub dev: i32,     // File system's disk device
    pub ino: u32,     // Inode number
    pub nlink: i16,   // Number of links to file
    pub size: u32,    // Size of file in bytes
}

pub const T_DIR: i16  = 1;
pub const T_FILE: i16 = 2;
pub const T_DEV: i16  = 3;
