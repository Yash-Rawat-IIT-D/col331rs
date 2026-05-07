#![allow(unused_parens)] // False positive from bitfield macro

use modular_bitfield::prelude::*;

// Segment Descriptor

#[bitfield]
#[repr(C, packed)]
#[derive(Clone, Copy, Default, Debug)]
pub struct SegDesc {
    lim_15_0: B16,      // Low bits of segment limit
    base_15_0: B16,     // Low bits of segment base address
    base_23_16: B8,     // Middle bits of segment base address
    seg_type: B4,       // Segment type (see STA_ constants)
    s: B1,              // 0 = system, 1 = application
    dpl: B2,            // Descriptor Privilege Level
    p: B1,              // Present
    lim_19_16: B4,      // High bits of segment limit
    avl: B1,            // Unused (available for software use)
    rsv1: B1,           // Reserved
    db: B1,             // 0 = 16-bit segment, 1 = 32-bit segment
    g: B1,              // Granularity: limit scaled by 4K when set
    base_31_24: B8,     // High bits of segment base address
}

impl SegDesc {
    /// Create a normal segment descriptor
    /// Matches the C macro: SEG(type, base, lim, dpl)
    pub fn seg(seg_type: u8, base: u32, lim: u32, dpl: u8) -> Self {
        let mut seg = SegDesc::default();
        seg.set_lim_15_0(((lim >> 12) & 0xffff) as u16);
        seg.set_base_15_0((base & 0xffff) as u16);
        seg.set_base_23_16(((base >> 16) & 0xff) as u8);
        seg.set_seg_type(seg_type);
        seg.set_s(1);
        seg.set_dpl(dpl);
        seg.set_p(1);
        seg.set_lim_19_16((lim >> 28) as u8);
        seg.set_avl(0);
        seg.set_rsv1(0);
        seg.set_db(1);
        seg.set_g(1);
        seg.set_base_31_24(((base >> 24) & 0xff) as u8);
        seg
    }
}
