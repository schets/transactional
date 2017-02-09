use std::sync::atomic::Ordering;

#[cfg(target_arch="x86_64")]
mod inner {
    use std::sync::atomic::Ordering;
    #[inline(always)]
    pub unsafe fn prefetchw(ptr: *const u8) {
        asm!("prefetchw ($0)"
         : : "r" (ptr) : "volatile")
    }
    pub const CONSUME: Ordering = Ordering::Relaxed;
}

#[cfg(target_arch="arm")]
mod inner {
    use std::sync::atomic::Ordering;
    #[inline(always)]
    pub unsafe fn prefetchw(ptr: *const u8) {
        asm!("pldw $0"
         : : "r" (ptr) : "volatile")
    }
    pub const CONSUME: Ordering = Ordering::Relaxed;
}

#[cfg(target_arch="aarch64")]
mod inner {
    use std::sync::atomic::Ordering;
    #[inline(always)]
    pub unsafe fn prefetchw(ptr: *const u8) {
        asm!("prfm PSTL1KEEP, $0"
         : : "r" (ptr) : "volatile")
    }
    pub const CONSUME: Ordering = Ordering::Relaxed;
}

#[cfg(any(target_arch = "powerpc", target_arch="powerpc64"))]
mod inner {
    use std::sync::atomic::Ordering;
    #[inline(always)]
    pub unsafe fn prefetchw2(ptr: *const u8) {
        asm!("dcbtst 0, $0"
         : : "r" (ptr) : "volatile")
    }
    pub const CONSUME: Ordering = Ordering::Relaxed;
}

#[cfg(not(any(target_arch="arm", target_arch="x86_64", target_arch="aarch64",
              target_arch="powerpc", target_arch="powerpc64")))]
mod inner {
    use std::sync::atomic::Ordering;
    #[inline(always)]
    pub unsafe fn prefetchw(ptr: *const u8) {}
    pub const CONSUME: Ordering = Ordering::Acquire;
}

#[inline(always)]
pub unsafe fn prefetchw<T>(pt: *const T) {
    inner::prefetchw(pt as *const u8);
}

pub const CONSUME: Ordering = inner::CONSUME;
