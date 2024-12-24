#![no_std]
extern crate alloc;
use alloc::vec::Vec;

use zerocopy::{error::AllocError, FromZeros};

/// Intentionally private, so that [VecZeroed] trait is sealed.
/// ```compile_fail
/// #[allow(unused_imports)]
/// use zerocopy_vec::Sealed;
/// ```
struct Seal;

/// Extension trait for [Vec], it adds [VecZeroed::extend_zeroed], [VecZeroed::insert_zeroed] and
/// [VecZeroed::new_zeroed] shortcut methods.
///
/// It's a sealed trait - not to be implemented outside this crate.
pub trait VecZeroed: Sized {
    // @TODO Docs
    /// Like [FromZeros::extend_vec_zeroed] and forwarding to it.
    fn extend_zeroed(&mut self, additional: usize) -> Result<(), AllocError>;

    // @TODO Docs
    /// Like [FromZeros::insert_vec_zeroed] and forwarding to it.
    fn insert_zeroed(&mut self, position: usize, additional: usize) -> Result<(), AllocError>;

    fn new_zeroed(len: usize) -> Result<Self, AllocError>;

    // @TODO Docs
    /// This function exists only to ensure that the trait is sealed.
    #[allow(private_interfaces)]
    fn sealed() -> Seal;
}

impl<T> VecZeroed for Vec<T>
where
    T: FromZeros,
{
    #[inline]
    fn extend_zeroed(&mut self, additional: usize) -> Result<(), AllocError> {
        <T as FromZeros>::extend_vec_zeroed(self, additional)
    }

    #[inline]
    fn insert_zeroed(&mut self, position: usize, additional: usize) -> Result<(), AllocError> {
        <T as FromZeros>::insert_vec_zeroed(self, position, additional)
    }

    #[inline]
    fn new_zeroed(len: usize) -> Result<Self, AllocError> {
        <T as FromZeros>::new_vec_zeroed(len)
    }

    #[allow(private_interfaces)]
    fn sealed() -> Seal {
        Seal
    }
}

// NOT fallible - but Linux VM overcommit-friendly.
#[macro_export]
macro_rules! vec_zeroed {
    ($T: ty; $GIVEN_LEN_T: expr) => {
        {
            // LEN_T exists to make errors clearer (if the user-provided expression is not const).
            const LEN_T: usize = $GIVEN_LEN_T;
            const ALIGN: ::core::primitive::usize = ::core::mem::align_of::<$T>();
            const SIZE_T: ::core::primitive::usize = ::core::mem::size_of::<$T>();

            const SIZE_PRIMITIVE: ::core::primitive::usize = SIZE_T/ALIGN * LEN_T;
            unsafe {
                match(ALIGN) {
                    0  => ::core::mem::transmute::<Vec<()>, Vec<$T>>(::alloc::vec![(); SIZE_PRIMITIVE]), // Unsure if this is lazily allocated by OS - do TEST!
                    1  => ::core::mem::transmute::<Vec<u8>, Vec<$T>>(::alloc::vec![0u8; SIZE_PRIMITIVE]),
                    2  => ::core::mem::transmute::<Vec<u16>, Vec<$T>>(::alloc::vec![0u16; SIZE_PRIMITIVE]),
                    4  => ::core::mem::transmute::<Vec<u32>, Vec<$T>>(::alloc::vec![0u32; SIZE_PRIMITIVE]),
                    8  => ::core::mem::transmute::<Vec<u64>, Vec<$T>>(::alloc::vec![0u64; SIZE_PRIMITIVE]),
                    16 => ::core::mem::transmute::<Vec<u128>, Vec<$T>>(::alloc::vec![0u128; SIZE_PRIMITIVE]),
                    other => unreachable!("Unsupported alignment: {}", other)
                }
            }
        }
    };
}

//--------
// Test-only. It's at the top level, so that `bytes()` can be at top level, too.`
#[cfg(test)]
const TERA: usize = 1024 * 1024 * 1024 * 1024;

/// For testing the call over FFI (no difference).
#[cfg(test)]
#[no_mangle]
#[allow(improper_ctypes_definitions)]
pub extern "C" fn bytes() -> Vec<u8> {
    extern crate alloc;
    use alloc::vec;

    vec![0u8; TERA]
}

#[cfg(test)]
pub fn bytes_extern() -> Vec<u8> {
    extern "C" {
        #[allow(improper_ctypes)]
        fn bytes() -> Vec<u8>;
    }
    unsafe { bytes() }
}

#[cfg(test)]
mod tests {
    use super::VecZeroed;
    use super::TERA;
    use alloc::vec;
    use alloc::vec::Vec;
    use core::hint;
    use zerocopy::{error::AllocError, FromZeros};

    #[test]
    fn extend_zero_by_zero() -> Result<(), AllocError> {
        let mut u8s = vec![0u8, 0u8];
        u8s.extend_zeroed(TERA)?;
        Ok(())
    }

    #[test]
    fn extend_zero_by_zero_original() -> Result<(), AllocError> {
        let mut u8s = vec![0u8, 0u8];
        FromZeros::extend_vec_zeroed(&mut u8s, TERA)?;
        Ok(())
    }
    //-----
    #[test]
    fn extend_zero_bulk() -> Result<(), AllocError> {
        let mut u8s = vec![0u8; 2];
        u8s.extend_zeroed(TERA)?;
        Ok(())
    }

    #[test]
    fn extend_zero_bulk_original() -> Result<(), AllocError> {
        let mut u8s = vec![0u8; 2];
        FromZeros::extend_vec_zeroed(&mut u8s, TERA)?;
        Ok(())
    }
    //------

    #[test]
    fn extend_zero_default_bulk_original() -> Result<(), AllocError> {
        let mut u8s: Vec<u8> = vec![Default::default(); 2];
        FromZeros::extend_vec_zeroed(&mut u8s, TERA)?;
        Ok(())
    }
    //-----

    #[test]
    fn extend_zeros_new_vec_both_original() -> Result<(), AllocError> {
        let mut u8s: Vec<u8> = u8::new_vec_zeroed(if true {
            0 // small non-zero size fails below, too
        } else {
            TERA
        })?;
        FromZeros::extend_vec_zeroed(&mut u8s, TERA)?;
        Ok(())
    }

    //-----
    #[test]
    fn extend_nonzeros() -> Result<(), AllocError> {
        let mut u8s = vec![0u8, 1u8, 2u8];
        u8s.extend_zeroed(TERA)?;
        Ok(())
    }

    #[test]
    fn extend_nonzeros_original() -> Result<(), AllocError> {
        let mut u8s = vec![0u8, 1u8, 2u8];
        FromZeros::extend_vec_zeroed(&mut u8s, TERA)?;
        Ok(())
    }
    //---------

    #[test]
    fn insert_nonzeros() -> Result<(), AllocError> {
        let mut u8s = vec![0u8, 1u8, 2u8];
        u8s.insert_zeroed(3, TERA)?;
        Ok(())
    }

    #[test]
    fn insert_nonzeroed_original() -> Result<(), AllocError> {
        let mut u8s = vec![0u8, 1u8, 2u8];
        FromZeros::insert_vec_zeroed(&mut u8s, 3, TERA)?;
        Ok(())
    }
    //-------

    #[test]
    fn insert_zeros_new_vec_both_original() -> Result<(), AllocError> {
        let mut u8s: Vec<u8> = u8::new_vec_zeroed(TERA)?;
        FromZeros::insert_vec_zeroed(&mut u8s, TERA - 1, TERA)?;
        Ok(())
    }
    //-------

    #[test]
    fn new_original() -> Result<(), AllocError> {
        let mut u8s = u8::new_vec_zeroed(TERA)?;
        Ok(())
    }

    #[test]
    fn insert_to_empty() -> Result<(), AllocError> {
        let mut u8s = Vec::<u8>::new();
        u8s.insert_zeroed(0, TERA)?;
        Ok(())
    }

    #[test]
    fn new_zeroed() -> Result<(), AllocError> {
        let u8s = Vec::<u8>::new_zeroed(TERA)?;
        // With the following it fails:
        if false {
            hint::black_box(u8s);
        }
        Ok(())
    }

    #[test]
    fn from_vec_macro() {
        let u8s = vec![0u8; TERA];
        // With the following enabled it fails - surprisingly. black_box must turn off allocation
        // flags:
        let mut u8s = if false { hint::black_box(u8s) } else { u8s };
        // Linux with default VM overcommit enabled, and with transparent huge pages turned off,
        // handles this well.
        // ```
        // cat /sys/kernel/mm/transparent_hugepage/enabled
        // always madvise [never]
        // ```
        // or:
        // ```
        // cat /sys/kernel/mm/transparent_hugepage/enabled
        // always [madvise] never
        // ```
        //
        // See also
        // https://stackoverflow.com/questions/71946484/why-is-vec0-super-large-number-so-memory-efficient-when-the-default-value.

        u8s[TERA - 1] = 1;

        // The following runs out of memory - even without black_box:
        if false {
            extern crate std;
            use std::println;
            println!("The modified last byte: {}", u8s[TERA - 1]);
        }

        // BUT, the following works well:
        hint::black_box(u8s[TERA - 1]);
    }

    #[test]
    fn from_extern() {
        use super::bytes_extern;

        // Calling through FFI doesn't change anything.
        let bytes = bytes_extern();
        hint::black_box(bytes[TERA - 1]);
    }

    #[test]
    fn vec_zeroed() {
        let mut u16s = vec_zeroed![u16; TERA];
        u16s[TERA - 1] = 2;

        let read = u16s[TERA - 1];
        // OK:
        hint::black_box(u16s[TERA - 1]);

        // The following runs out of memory - even without black_box:
        if false {
            extern crate std;
            std::println!("The modified word: {read}.");
        }
    }

    #[test]
    fn vec_default() {
        #[derive(Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
        struct S {
            first: u8,
            second: u8,
        }

        // Either of the following is OK:
        let mut ses: Vec<S> = if true {
            vec![Default::default(); TERA]
        } else {
            vec![S::default(); TERA]
        };

        ses[TERA - 1] = S {
            first: 1,
            second: 2,
        };
        hint::black_box(ses[TERA - 1]);
        if false {
            extern crate std;
            std::println!("The modified word: {:?}.", ses[TERA - 1]);
        }
    }
}
