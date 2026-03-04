// Mutual exclusion spin locks and debugging utilities

/// Record the current call stack in pcs[] by following the %ebp chain.
/// This function walks the stack frame pointers to collect return addresses.
pub fn getcallerpcs(v: *const u32, pcs: &mut [u32; 10]) {
    unsafe {
        let mut ebp = v.offset(-2) as *const u32;
        let mut i = 0;
        
        while i < 10 {
            // Check for invalid ebp values
            // if(ebp == 0 || ebp < (uint*)KERNBASE || ebp == (uint*)0xffffffff)
            if ebp.is_null() || ebp == 0xffffffff as *const u32 {
                break;
            }
            
            // pcs[i] = ebp[1]; // saved %eip
            pcs[i] = *ebp.offset(1);
            
            // ebp = (uint*)ebp[0]; // saved %ebp
            ebp = *ebp as *const u32;
            
            i += 1;
        }
        
        // Fill rest with zeros
        while i < 10 {
            pcs[i] = 0;
            i += 1;
        }
    }
}
