use crate::mp::MP_ONCE;  // Import the MP_ONCE static from mp.rs
// use core::ptr;
use crate::constants::NSEGS;
use crate::mmu::SegDesc;

#[derive(Debug, Clone, Copy)]
pub struct Cpu {
    pub apicid: u8,  // Local APIC ID
    pub gdt: [SegDesc; NSEGS],   // x86 global descriptor table
}

impl Cpu {
    pub const fn new() -> Self {
        Self { 
            apicid: 0,
            gdt: [SegDesc::new(); NSEGS],
        }
    }
}

pub fn cpuid() -> usize {
    let cpus = MP_ONCE.cpus.get().expect("CPUs not initialized");
    unsafe { (mycpu() as *const Cpu).offset_from(cpus.as_ptr()) as usize }
}

pub fn mycpu() -> &'static Cpu {
    let cpus = MP_ONCE.cpus.get().expect("CPUs not initialized");
    &cpus[0]
}