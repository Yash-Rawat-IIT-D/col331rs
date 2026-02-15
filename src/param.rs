pub const KSTACKSIZE: usize = 4096; // size of per-process kernel stack
pub const NCPU: usize = 8; // maximum number of CPUs
pub const NINODE: usize = 50; // maximum number of active i-nodes
pub const ROOTDEV: u32 = 1; // device number of file system root disk
pub const MAXOPBLOCKS: usize = 10; // max # of blocks any FS op writes
pub const LOGSIZE: usize = MAXOPBLOCKS * 3; // max data blocks in on-disk log
