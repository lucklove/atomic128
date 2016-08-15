#![feature(asm)]

#[derive(Clone, Copy, Debug)]
pub struct AtomicU128 {
    pub lo: u64,
    pub hi: u64,
}

#[cfg(target_arch = "x86_64")]
fn cas128(src: &AtomicU128, cmp: &mut AtomicU128, with: AtomicU128) -> bool {
    let result: bool;
    unsafe {
        asm!("
                lock cmpxchg16b $3
                setz $0
            "
             : "=q"(result), "+{rdx}"(cmp.hi), "+{rax}"(cmp.lo)
             : "*m"(src), "{rcx}"(with.hi), "{rbx}"(with.lo)
             : "memory" "cc"
        );
    }
    result
}

impl AtomicU128 {
    pub fn new(lo: u64, hi: u64) -> Self {
        AtomicU128 { lo: lo, hi: hi }
    }

    pub fn zero() -> Self {
        AtomicU128 { lo: 0, hi: 0 }
    }

    pub fn load(&self) -> Self {
        let mut ret = Self::zero();
        cas128(self, &mut ret, Self::zero());
        ret
    }

    pub fn store(&self, val: AtomicU128) {
        let mut current = Self::zero();
        while !cas128(self, &mut current, val) {
            // Loop until success
        }
    }

    pub fn swap(&self, val: AtomicU128) -> AtomicU128 {
        let mut prev = Self::zero();
        while !cas128(self, &mut prev, val) {
            // Loop until success
        }
        prev
    }

    pub fn compare_and_swap(&self, current: AtomicU128, new: AtomicU128) -> AtomicU128 {
        let mut current = current;
        cas128(self, &mut current, new);
        current
    }

    pub fn compare_exchange(&self, current: AtomicU128, new: AtomicU128) -> Result<AtomicU128, AtomicU128> {
        let mut current = current;
        if cas128(self, &mut current, new) {
            Ok(current)
        } else {
            Err(current)
        }
    }

    pub fn compare_exchange_weak(&self, current: AtomicU128, new: AtomicU128) -> Result<AtomicU128, AtomicU128> {
        self.compare_exchange(current, new)    
    }
}

impl std::cmp::PartialEq for AtomicU128 {
    fn eq(&self, other: &Self) -> bool {
        self.lo == other.lo && self.hi == other.hi
    }
}

impl std::default::Default for AtomicU128 {
    fn default() -> Self {
        Self::zero()
    }
} 

unsafe impl Sync for AtomicU128 {}

#[cfg(test)]
mod tests {
    use super::{AtomicU128, cas128};

    #[test]
    fn test_cas_success() {
        let mut a = AtomicU128::new(1, 2);
        let mut b = a;
        let c = AtomicU128::new(2, 3);
        assert_eq!(cas128(&mut a, &mut b, c), true);
        assert_eq!(a.lo, 2);
        assert_eq!(a.hi, 3);
    }

    #[test]
    fn test_cas_failure() {
        let mut a = AtomicU128::new(1, 2);
        let mut b = AtomicU128::new(2, 3);
        let c = AtomicU128::zero();
        assert_eq!(cas128(&mut a, &mut b, c), false);
        assert_eq!(b.lo, 1);
        assert_eq!(b.hi, 2);
    }

    #[test]
    fn test_load() {
        let a = AtomicU128::new(1, 2);
        let b = a.load();
        assert_eq!(a, b);
    }

    #[test]
    fn test_store() {
        let a = AtomicU128::zero();
        let b = AtomicU128::new(2, 3);
        a.store(b);
        assert_eq!(a, b);
    }

    #[test]
    fn test_swap() {
        let a = AtomicU128::new(2, 3);
        let mut b = AtomicU128::new(4, 5);
        b = a.swap(b);
        assert_eq!(a, AtomicU128::new(4, 5));
        assert_eq!(b, AtomicU128::new(2, 3));
    }

    #[test]
    fn test_compare_and_swap() {
        let a = AtomicU128::new(1, 1);
        assert_eq!(a.compare_and_swap(a, AtomicU128::zero()), AtomicU128::new(1, 1));
    }

    #[test]
    fn test_compare_exchange_weak() {
        let a = AtomicU128::new(1, 1);
        let b = AtomicU128::new(1, 2);
        assert_eq!(a.compare_exchange_weak(b, b), Err(a));
        assert_eq!(a.compare_exchange_weak(a, b), Ok(AtomicU128::new(1, 1)));
    }
}