use modular_bitfield::prelude::*;
use core::cell::{OnceCell, RefCell};
use crate::proc::cpuid;
use crate::println;
use crate::lapic::lapiceoi;
use crate::x86::{lidt, rcr2, TrapFrame};
use crate::lapic;
use crate::constants::{SEG_KCODE, STS_IG32, STS_TG32, T_IRQ0, IRQ_TIMER, IRQ_ERROR, IRQ_SPURIOUS};

extern "C" {
    static vectors: [usize; 256]; // remove assembly. 
}

#[bitfield]
#[repr(C, packed)]
#[derive(Clone, Copy, Default)] // debug can be removed ? do we need to ? 
pub struct GateDesc {
    off_15_0: B16,   // low 16 bits of offset in segment
    cs: B16,         // code segment selector
    args: B5,        // # args (0 for interrupt/trap gates)
    rsv1: B3,        // reserved bits (should be zero)
    r_type: B4,      // type (e.g., STS_IG32, STS_TG32)
    s: B1,         // must be 0 (system)
    dpl: B2,         // descriptor privilege level
    p: B1,         // present
    off_31_16: B16,  // high bits of offset in segment
}
impl GateDesc {
    pub fn set_gate(&mut self, is_trap: bool, sel: u16, off: usize, dpl: u8) {
        self.set_off_15_0((off & 0xffff) as u16);
        self.set_cs(sel);
        self.set_args(0);
        self.set_rsv1(0);
        let typ = if is_trap { STS_TG32 } else { STS_IG32 };
        self.set_r_type(typ);  // for an interrupt gate, for example.
        self.set_s(0);
        self.set_dpl(dpl);
        self.set_p(1);
        self.set_off_31_16((off >> 16) as u16);
    }
}


#[repr(C)]
pub struct IDTOnce {
    pub idt: OnceCell<[GateDesc; 256]>,
}  
unsafe impl Sync for IDTOnce {}
pub static IDT: IDTOnce = IDTOnce { idt: OnceCell::new() };

pub struct TickCounter {
    ticks: RefCell<u32>
}
unsafe impl Sync for TickCounter {}
static TICKS: TickCounter = TickCounter { ticks: RefCell::new(0) };


pub fn tvinit() {
    let mut arr = [GateDesc::default(); 256];
    for i in 0..256 {
        arr[i].set_gate(
            false,                  // Use an interrupt gate.
            SEG_KCODE << 3,         // Code segment selector (shifted as in the C code).
            unsafe { vectors[i] },  // Offset from the external vector table.
            0                       // Descriptor privilege level.
        );
    }
    IDT.idt.set(arr);
}

pub fn idtinit() {
    lidt(IDT.idt.get().unwrap() , core::mem::size_of::<[GateDesc; 256]>() as usize); 
}


#[no_mangle]
pub extern "C" fn trap(orig_tf: *mut TrapFrame) {

    if orig_tf.is_null() {
        panic!("Received null TrapFrame pointer");
    }

    let tf = unsafe { &mut *orig_tf };

	const TIMER: u32 = T_IRQ0 + IRQ_TIMER;
	const SPURIOUS: u32 = T_IRQ0 + IRQ_SPURIOUS;
	const SEVEN: u32 = T_IRQ0 + 7;
	
    match tf.trapno {
		TIMER => {
            *TICKS.ticks.borrow_mut() += 1;
            println!("Tick {}!", TICKS.ticks.borrow());
			lapic::lapiceoi();
		}
		SEVEN | SPURIOUS => {
			println!(
				"cpu{}: spurious interrupt at {}:{}\n",
				cpuid() ,
				tf.cs,
				tf.eip
			);
            lapiceoi();
		}
		_ => {
			println!(
				"unexpected trap {} from cpu {} eip {} (cr2=0x{:x})\n",
				tf.trapno,
				cpuid(),
				tf.eip,
				rcr2()
			);
			panic!("trap happened");
		}
	}
}