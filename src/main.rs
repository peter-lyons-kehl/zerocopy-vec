extern crate alloc;
use alloc::vec::Vec;
use zerocopy::{error::AllocError, FromZeros};

fn main() -> Result<(), AllocError> {
    // If used as number of bytes, it's 1TB.
    const TERA: usize = 1024 * 1024 * 1024 * 1024;

    if false {
        let mut u8s: Vec<u8> = u8::new_vec_zeroed(if true {
            0 // small non-zero size fails below, too
        } else {
            TERA
        })?;
        if true {
            FromZeros::extend_vec_zeroed(&mut u8s, TERA)?;
        }
    }

    if true {
        let mut u8s: Vec<u8> = u8::new_vec_zeroed(TERA)?;
        if true {
            FromZeros::insert_vec_zeroed(&mut u8s, TERA - 1, TERA)?;
        }
    }
    Ok(())
}
