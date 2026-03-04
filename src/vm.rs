use crate::constants::{SEG_KCODE, SEG_KDATA, SEG_UCODE, SEG_UDATA, STA_X, STA_W, STA_R, STARTPROC, PROCSIZE};
use crate::mmu::SegDesc;
use crate::proc::cpuid;
use crate::mp::MP_ONCE;
use crate::x86::lgdt;
use core::mem::size_of_val;

/// Set up CPU's kernel segment descriptors.
/// Run once on entry on each CPU.
pub fn seginit() {
    unsafe {
        // Map "logical" addresses to virtual addresses using identity map.
        let cpus = MP_ONCE.cpus.get().expect("CPUs not initialized");
        let cpu_ptr = cpus.as_ptr() as *mut crate::proc::Cpu;
        let c = &mut *cpu_ptr.add(cpuid());
        
        c.gdt[SEG_KCODE as usize] = SegDesc::seg(STA_X | STA_R, 0, 0xffffffff, 0);
        c.gdt[SEG_KDATA as usize] = SegDesc::seg(STA_W, 0, 0xffffffff, 0);
        c.gdt[SEG_UCODE as usize] = SegDesc::seg(STA_X | STA_R, STARTPROC, PROCSIZE, 0);
        c.gdt[SEG_UDATA as usize] = SegDesc::seg(STA_W, STARTPROC, PROCSIZE, 0);
        lgdt(&c.gdt, size_of_val(&c.gdt));
    }
}
