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
    fn extend_zeroed(&mut self, additional: usize) -> Result<(), AllocError> {
        <T as FromZeros>::extend_vec_zeroed(self, additional)
    }

    fn insert_zeroed(&mut self, position: usize, additional: usize) -> Result<(), AllocError> {
        <T as FromZeros>::insert_vec_zeroed(self, position, additional)
    }

    fn new_zeroed(len: usize) -> Result<Self, AllocError> {
        <T as FromZeros>::new_vec_zeroed(len)
    }

    #[allow(private_interfaces)]
    fn sealed() -> Seal {
        Seal
    }
}

#[cfg(test)]
mod tests {
    use super::VecZeroed;
    use alloc::vec;
    use alloc::vec::Vec;
    use core::hint;
    use zerocopy::error::AllocError;

    const TERA: usize = 1024 * 1024 * 1024 * 1024;

    #[test]
    fn extend() -> Result<(), AllocError> {
        let mut u8s = vec![0u8, 1u8, 2u8];
        u8s.extend_zeroed(TERA)?;
        Ok(())
    }

    #[test]
    fn insert() -> Result<(), AllocError> {
        let mut u8s = vec![0u8, 1u8, 2u8];
        u8s.insert_zeroed(3, TERA)?;
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
        let u8s = Vec::<u8>::new_zeroed(1_000_000_000_000)?;
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
        // handles this well:
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
}
